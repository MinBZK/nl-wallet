package nl.rijksoverheid.edi.wallet.platform_support.close_proximity_disclosure

import android.content.Context
import androidx.annotation.VisibleForTesting
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import org.multipaz.cbor.Bstr
import org.multipaz.cbor.Cbor
import org.multipaz.cbor.Simple
import org.multipaz.cbor.Tagged
import org.multipaz.cbor.buildCborArray
import org.multipaz.context.initializeApplication
import org.multipaz.crypto.Crypto
import org.multipaz.crypto.EcCurve
import org.multipaz.mdoc.connectionmethod.MdocConnectionMethod
import org.multipaz.mdoc.connectionmethod.MdocConnectionMethodBle
import org.multipaz.mdoc.engagement.buildDeviceEngagement
import org.multipaz.mdoc.role.MdocRole
import org.multipaz.mdoc.sessionencryption.SessionEncryption
import org.multipaz.mdoc.transport.MdocTransport
import org.multipaz.mdoc.transport.MdocTransportFactory
import org.multipaz.mdoc.transport.MdocTransportOptions
import org.multipaz.mdoc.transport.advertise
import org.multipaz.mdoc.transport.waitForConnection
import org.multipaz.util.Constants
import org.multipaz.util.UUID
import org.multipaz.util.toBase64Url
import nl.rijksoverheid.edi.wallet.platform_support.PlatformSupportInitializer
import uniffi.platform_support.CloseProximityDisclosureChannel
import uniffi.platform_support.CloseProximityDisclosureException
import uniffi.platform_support.CloseProximityDisclosureUpdate
import uniffi.platform_support.CloseProximityDisclosureBridge as RustCloseProximityDisclosureBridge

/**
 * This class is automatically initialized on app start through
 * the [PlatformSupportInitializer] class.
 */
@OptIn(ExperimentalUnsignedTypes::class)
class CloseProximityDisclosureBridge(
    val context: Context,
    private val testingPeripheralServerModeUuid: UUID? = null,
) : RustCloseProximityDisclosureBridge {
    private class StopObservingTransportState : CancellationException()

    private class ActiveSession(
        val channel: CloseProximityDisclosureChannel,
        val transports: List<MdocTransport>,
    )

    private val bridgeScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    // UniFFI/Rust can reach these suspend bridge methods from unrelated coroutines. Serialize
    // start/stop transitions so rapid start/stop/start calls are deterministic.
    private val lifecycleMutex = Mutex()
    // Background work only acts on the session it was created for. Identity checks against the
    // current activeSession keep stale work from a replaced session from emitting updates after
    // a newer handover has already started.
    private val activeSessionMutex = Mutex()
    private var activeSession: ActiveSession? = null

    init {
        initializeApplication(context.applicationContext)
    }

    override suspend fun startQrHandover(channel: CloseProximityDisclosureChannel): String =
        lifecycleMutex.withLock {
            stopBleServerLocked()

            try {
                val eDeviceKey = Crypto.createEcPrivateKey(EcCurve.P256)
                val connectionMethod = MdocConnectionMethodBle(
                    supportsPeripheralServerMode = true,
                    supportsCentralClientMode = false,
                    peripheralServerModeUuid = testingPeripheralServerModeUuid ?: UUID.randomUUID(),
                    centralClientModeUuid = null,
                    peripheralServerModePsm = null,
                    // Android does not expose the local BLE MAC address in a stable way on modern devices.
                    peripheralServerModeMacAddress = null,
                )
                val advertisedTransports = listOf<MdocConnectionMethod>(connectionMethod).advertise(
                    role = MdocRole.MDOC,
                    transportFactory = MdocTransportFactory.Default,
                    options = MdocTransportOptions(),
                )
                val encodedDeviceEngagement = Cbor.encode(
                    buildDeviceEngagement(eDeviceKey = eDeviceKey.publicKey) {
                        advertisedTransports.forEach { addConnectionMethod(it.connectionMethod) }
                    }.toDataItem()
                )
                val qrCode = encodedDeviceEngagement.toBase64Url()
                val session = ActiveSession(channel = channel, transports = advertisedTransports)

                activeSessionMutex.withLock {
                    activeSession = session
                }

                bridgeScope.launch {
                    try {
                        val transport = advertisedTransports.waitForConnection(eSenderKey = eDeviceKey.publicKey)
                        // A newer start/stop may already have replaced this session while the
                        // connection attempt was in flight. If that happened, close the transport
                        // we obtained and exit without reporting more updates.
                        if (!isSessionActive(session)) {
                            // Multipaz uses close() as the cancellation path for the transport that won the race
                            // after the caller no longer wants it:
                            // https://github.com/openwallet-foundation/multipaz/blob/c89f66131e65b7964cdf0fc2961d17cec2b7d781/multipaz/src/commonMain/kotlin/org/multipaz/mdoc/transport/connectionHelper.kt#L75-L123
                            // https://github.com/openwallet-foundation/multipaz/blob/c89f66131e65b7964cdf0fc2961d17cec2b7d781/multipaz/src/commonMain/kotlin/org/multipaz/mdoc/transport/BleTransportPeripheralMdoc.kt#L206-L214
                            transport.close()
                            return@launch
                        }
                        channel.sendUpdate(update = CloseProximityDisclosureUpdate.Connecting)

                        bridgeScope.launch {
                            observeTransportState(session = session, transport = transport)
                        }

                        receiveMessages(
                            session = session,
                            transport = transport,
                            eDeviceKey = eDeviceKey,
                            encodedDeviceEngagement = encodedDeviceEngagement,
                        )
                    } catch (exception: Exception) {
                        failSession(session = session, exception = exception)
                    }
                }

                qrCode
            } catch (exception: Exception) {
                runCatching {
                    channel.sendUpdate(
                        update = CloseProximityDisclosureUpdate.Error(
                            error = exception.asPlatformError(),
                        ),
                    )
                }
                throw exception.asPlatformError()
            }
        }

    override suspend fun sendDeviceResponse(deviceResponse: List<UByte>) {}

    override suspend fun stopBleServer() {
        lifecycleMutex.withLock {
            stopBleServerLocked()
        }
    }

    private suspend fun stopBleServerLocked() {
        val session = activeSessionMutex.withLock {
            // Clear the session first so already-running background coroutines immediately observe
            // themselves as stale before transport.close() wakes them up.
            activeSession?.also { activeSession = null }
        } ?: return

        closeSessionTransports(session)
        runCatching { session.channel.sendUpdate(update = CloseProximityDisclosureUpdate.Closed) }
    }

    @VisibleForTesting
    suspend fun isBleServerActiveForTesting(): Boolean =
        activeSessionMutex.withLock {
            activeSession != null
        }

    private suspend fun isSessionActive(session: ActiveSession): Boolean =
        activeSessionMutex.withLock {
            activeSession === session
        }

    private suspend fun clearSession(session: ActiveSession): Boolean =
        activeSessionMutex.withLock {
            if (activeSession === session) {
                activeSession = null
                true
            } else {
                false
            }
        }

    private suspend fun createSessionEncryption(
        eDeviceKey: org.multipaz.crypto.EcPrivateKey,
        encodedDeviceEngagement: ByteArray,
        message: ByteArray,
    ): Pair<SessionEncryption, ByteArray> {
        val eReaderKey = SessionEncryption.getEReaderKey(message)
        val encodedSessionTranscript = Cbor.encode(
            buildCborArray {
                add(
                    Tagged(
                        Tagged.ENCODED_CBOR,
                        Bstr(encodedDeviceEngagement),
                    ),
                )
                add(
                    Tagged(
                        Tagged.ENCODED_CBOR,
                        Bstr(eReaderKey.encodedCoseKey),
                    ),
                )
                add(Simple.NULL)
            },
        )
        val sessionEncryption = SessionEncryption(
            role = MdocRole.MDOC,
            eSelfKey = eDeviceKey,
            remotePublicKey = eReaderKey.publicKey,
            encodedSessionTranscript = encodedSessionTranscript,
        )
        return sessionEncryption to encodedSessionTranscript
    }

    private suspend fun observeTransportState(
        session: ActiveSession,
        transport: MdocTransport,
    ) {
        try {
            transport.state.collect { state ->
                if (!isSessionActive(session)) {
                    throw StopObservingTransportState()
                }

                when (state) {
                    MdocTransport.State.CONNECTED -> {
                        session.channel.sendUpdate(update = CloseProximityDisclosureUpdate.Connected)
                    }
                    MdocTransport.State.CLOSED -> {
                        finishSession(session = session, update = CloseProximityDisclosureUpdate.Closed)
                        throw StopObservingTransportState()
                    }
                    MdocTransport.State.FAILED -> throw StopObservingTransportState()
                    else -> Unit
                }
            }
        } catch (_: StopObservingTransportState) {
        } catch (exception: Exception) {
            failSession(session = session, exception = exception)
        }
    }

    private suspend fun receiveMessages(
        session: ActiveSession,
        transport: MdocTransport,
        eDeviceKey: org.multipaz.crypto.EcPrivateKey,
        encodedDeviceEngagement: ByteArray,
    ) {
        var sessionEncryption: SessionEncryption? = null
        var encodedSessionTranscript: ByteArray? = null

        while (isSessionActive(session)) {
            // Multipaz explicitly documents and implements close() from another coroutine as the way to
            // interrupt waitForMessage():
            // https://github.com/openwallet-foundation/multipaz/blob/c89f66131e65b7964cdf0fc2961d17cec2b7d781/multipaz/src/commonMain/kotlin/org/multipaz/mdoc/transport/MdocTransportClosedException.kt#L3-L5
            // https://github.com/openwallet-foundation/multipaz/blob/c89f66131e65b7964cdf0fc2961d17cec2b7d781/multipaz/src/commonMain/kotlin/org/multipaz/mdoc/transport/BleTransportPeripheralMdoc.kt#L143-L160
            val message = transport.waitForMessage()
            if (!isSessionActive(session)) {
                return
            }

            if (message.isEmpty()) {
                finishSession(session = session, update = CloseProximityDisclosureUpdate.Closed)
                return
            }

            if (sessionEncryption == null || encodedSessionTranscript == null) {
                val sessionState = createSessionEncryption(
                    eDeviceKey = eDeviceKey,
                    encodedDeviceEngagement = encodedDeviceEngagement,
                    message = message,
                )
                sessionEncryption = sessionState.first
                encodedSessionTranscript = sessionState.second

                if (!isSessionActive(session)) {
                    transport.close()
                    return
                }
            }

            val (deviceRequest, status) = sessionEncryption!!.decryptMessage(message)
            if (deviceRequest == null && status == null) {
                throw CloseProximityDisclosureException.PlatformException(
                    reason = "Reader message did not contain a device request or status",
                )
            }

            if (deviceRequest != null) {
                session.channel.sendUpdate(
                    update = CloseProximityDisclosureUpdate.SessionEstablished(
                        sessionTranscript = encodedSessionTranscript!!.toUByteList(),
                        deviceRequest = deviceRequest.toUByteList(),
                    ),
                )
            }

            if (status != null) {
                finishSession(
                    session = session,
                    update = status.asCloseProximityDisclosureUpdate(),
                )
                return
            }
        }
    }

    private suspend fun finishSession(
        session: ActiveSession,
        update: CloseProximityDisclosureUpdate,
    ) {
        val shouldReport = clearSession(session)
        closeSessionTransports(session)
        if (shouldReport) {
            runCatching { session.channel.sendUpdate(update = update) }
        }
    }

    private suspend fun failSession(
        session: ActiveSession,
        exception: Exception,
    ) {
        val shouldReport = clearSession(session)
        closeSessionTransports(session)
        if (shouldReport) {
            runCatching {
                session.channel.sendUpdate(
                    update = CloseProximityDisclosureUpdate.Error(
                        error = exception.asPlatformError(),
                    ),
                )
            }
        }
    }

    private suspend fun closeSessionTransports(session: ActiveSession) {
        // Multipaz expects transport.close() to tear down in-flight BLE work: the transport cancels
        // its current job and the Android peripheral manager closes incomingMessages, which wakes
        // waitForMessage() and lets replaced sessions die without reattaching to the new one.
        // https://github.com/openwallet-foundation/multipaz/blob/c89f66131e65b7964cdf0fc2961d17cec2b7d781/multipaz/src/commonMain/kotlin/org/multipaz/mdoc/transport/BleTransportPeripheralMdoc.kt#L206-L214
        // https://github.com/openwallet-foundation/multipaz/blob/c89f66131e65b7964cdf0fc2961d17cec2b7d781/multipaz/src/androidMain/kotlin/org/multipaz/mdoc/transport/BlePeripheralManagerAndroid.kt#L567-L579
        session.transports.forEach { runCatching { it.close() } }
    }

    private fun ByteArray.toUByteList(): List<UByte> = asUByteArray().toList()

    private fun Long.asCloseProximityDisclosureUpdate(): CloseProximityDisclosureUpdate =
        when (this) {
            Constants.SESSION_DATA_STATUS_SESSION_TERMINATION -> CloseProximityDisclosureUpdate.Closed
            Constants.SESSION_DATA_STATUS_ERROR_SESSION_ENCRYPTION -> CloseProximityDisclosureUpdate.Error(
                error = CloseProximityDisclosureException.PlatformException(
                    reason = "Reader terminated the session with status 10 (session encryption error)",
                ),
            )
            Constants.SESSION_DATA_STATUS_ERROR_CBOR_DECODING -> CloseProximityDisclosureUpdate.Error(
                error = CloseProximityDisclosureException.PlatformException(
                    reason = "Reader terminated the session with status 11 (CBOR decoding error)",
                ),
            )
            else -> CloseProximityDisclosureUpdate.Error(
                error = CloseProximityDisclosureException.PlatformException(
                    reason = "Reader terminated the session with unexpected status $this",
                ),
            )
        }

    private fun Exception.asPlatformError(): CloseProximityDisclosureException =
        when (this) {
            is CloseProximityDisclosureException -> this
            else -> CloseProximityDisclosureException.PlatformException(
                reason = message ?: this::class.java.simpleName,
            )
        }
}
