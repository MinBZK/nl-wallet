package nl.rijksoverheid.edi.wallet.platform_support.close_proximity_disclosure

import android.content.Context
import androidx.annotation.VisibleForTesting
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancelAndJoin
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

    private data class SessionState(
        val sessionEncryption: SessionEncryption?,
        val encodedSessionTranscript: ByteArray?,
    )

    private data class EstablishedSessionContext(
        val transport: MdocTransport,
        val sessionEncryption: SessionEncryption,
    )

    private data class ReaderSessionContext(
        val sessionEncryption: SessionEncryption,
        val encodedSessionTranscript: ByteArray,
    )

    private class ActiveSession(
        val channel: CloseProximityDisclosureChannel,
        val transports: List<MdocTransport>,
        val eDeviceKey: org.multipaz.crypto.EcPrivateKey,
        val encodedDeviceEngagement: ByteArray,
    ) {
        private val sessionMutex = Mutex()
        private var readJob: Job? = null
        private var transport: MdocTransport? = null
        private var sessionEncryption: SessionEncryption? = null
        private var encodedSessionTranscript: ByteArray? = null

        suspend fun setReadJob(job: Job) {
            sessionMutex.withLock {
                readJob = job
            }
        }

        suspend fun cancelReadJobAndJoin() {
            val job = sessionMutex.withLock {
                readJob?.also { readJob = null }
            }
            job?.cancelAndJoin()
        }

        suspend fun setTransport(transport: MdocTransport) {
            sessionMutex.withLock {
                this.transport = transport
            }
        }

        suspend fun transport(): MdocTransport? =
            sessionMutex.withLock { transport }

        suspend fun sessionState(): SessionState =
            sessionMutex.withLock {
                SessionState(
                    sessionEncryption = sessionEncryption,
                    encodedSessionTranscript = encodedSessionTranscript,
                )
            }

        suspend fun establishedTransportAndEncryption(): EstablishedSessionContext? =
            sessionMutex.withLock {
                val currentTransport = transport ?: return@withLock null
                val currentSessionEncryption = sessionEncryption ?: return@withLock null
                EstablishedSessionContext(
                    transport = currentTransport,
                    sessionEncryption = currentSessionEncryption,
                )
            }

        suspend fun setSessionEncryption(
            sessionEncryption: SessionEncryption,
            encodedSessionTranscript: ByteArray,
        ) {
            sessionMutex.withLock {
                this.sessionEncryption = sessionEncryption
                this.encodedSessionTranscript = encodedSessionTranscript
            }
        }
    }

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
            startQrHandoverLocked(channel)
        }

    private suspend fun startQrHandoverLocked(channel: CloseProximityDisclosureChannel): String {
        stopBleServerLocked()

        return try {
            val session = createSession(channel)
            setActiveSession(session)
            waitForConnectionInBackground(session)
            session.encodedDeviceEngagement.toBase64Url()
        } catch (exception: Exception) {
            reportStartQrHandoverFailure(channel = channel, exception = exception)
            throw exception.asPlatformError()
        }
    }

    private suspend fun createSession(channel: CloseProximityDisclosureChannel): ActiveSession {
        val eDeviceKey = Crypto.createEcPrivateKey(EcCurve.P256)
        val advertisedTransports = advertiseTransports(buildBleConnectionMethod())
        val encodedDeviceEngagement = createEncodedDeviceEngagement(
            eDeviceKey = eDeviceKey,
            advertisedTransports = advertisedTransports,
        )
        return ActiveSession(
            channel = channel,
            transports = advertisedTransports,
            eDeviceKey = eDeviceKey,
            encodedDeviceEngagement = encodedDeviceEngagement,
        )
    }

    private fun buildBleConnectionMethod(): MdocConnectionMethodBle =
        MdocConnectionMethodBle(
            supportsPeripheralServerMode = true,
            supportsCentralClientMode = false,
            peripheralServerModeUuid = testingPeripheralServerModeUuid ?: UUID.randomUUID(),
            centralClientModeUuid = null,
            peripheralServerModePsm = null,
            // Android does not expose the local BLE MAC address in a stable way on modern devices.
            peripheralServerModeMacAddress = null,
        )

    private suspend fun advertiseTransports(
        connectionMethod: MdocConnectionMethod,
    ): List<MdocTransport> =
        listOf<MdocConnectionMethod>(connectionMethod).advertise(
            role = MdocRole.MDOC,
            transportFactory = MdocTransportFactory.Default,
            options = MdocTransportOptions(),
        )

    private fun createEncodedDeviceEngagement(
        eDeviceKey: org.multipaz.crypto.EcPrivateKey,
        advertisedTransports: List<MdocTransport>,
    ): ByteArray =
        Cbor.encode(
            buildDeviceEngagement(eDeviceKey = eDeviceKey.publicKey) {
                advertisedTransports.forEach { addConnectionMethod(it.connectionMethod) }
            }.toDataItem()
        )

    private suspend fun setActiveSession(session: ActiveSession) {
        activeSessionMutex.withLock {
            activeSession = session
        }
    }

    private fun waitForConnectionInBackground(session: ActiveSession) {
        bridgeScope.launch {
            try {
                val transport = session.transports.waitForConnection(eSenderKey = session.eDeviceKey.publicKey)
                handleConnectedTransport(session = session, transport = transport)
            } catch (exception: Exception) {
                failSession(session = session, exception = exception)
            }
        }
    }

    private suspend fun handleConnectedTransport(
        session: ActiveSession,
        transport: MdocTransport,
    ) {
        if (!isSessionActive(session)) {
            closeStaleTransport(transport)
            return
        }
        session.channel.sendUpdate(update = CloseProximityDisclosureUpdate.Connecting)
        session.setTransport(transport)
        bridgeScope.launch {
            observeTransportState(session = session, transport = transport)
        }
        startReadJob(session)
    }

    private suspend fun closeStaleTransport(transport: MdocTransport) {
        // Multipaz uses close() as the cancellation path for the transport that won the race
        // after the caller no longer wants it:
        // https://github.com/openwallet-foundation/multipaz/blob/c89f66131e65b7964cdf0fc2961d17cec2b7d781/multipaz/src/commonMain/kotlin/org/multipaz/mdoc/transport/connectionHelper.kt#L75-L123
        // https://github.com/openwallet-foundation/multipaz/blob/c89f66131e65b7964cdf0fc2961d17cec2b7d781/multipaz/src/commonMain/kotlin/org/multipaz/mdoc/transport/BleTransportPeripheralMdoc.kt#L206-L214
        transport.close()
    }

    private suspend fun reportStartQrHandoverFailure(
        channel: CloseProximityDisclosureChannel,
        exception: Exception,
    ) {
        runCatching {
            channel.sendUpdate(
                update = CloseProximityDisclosureUpdate.Error(
                    error = exception.asPlatformError(),
                ),
            )
        }
    }

    override suspend fun sendDeviceResponse(deviceResponse: List<UByte>) {
        val session = requireActiveSession()
        requireSessionActive(session)
        val establishedSessionContext = requireEstablishedSession(session)
        session.cancelReadJobAndJoin()
        requireSessionActive(session)
        sendTerminatingDeviceResponse(session, establishedSessionContext, deviceResponse)
        closeSessionAfterDeviceResponse(session)
    }

    private suspend fun requireActiveSession(): ActiveSession =
        activeSessionMutex.withLock { activeSession }
            ?: throw CloseProximityDisclosureException.PlatformException(
                reason = "No active close proximity disclosure session",
            )

    private suspend fun requireSessionActive(session: ActiveSession) {
        if (!isSessionActive(session)) {
            throw CloseProximityDisclosureException.PlatformException(
                reason = "Close proximity disclosure session is no longer active",
            )
        }
    }

    private suspend fun requireEstablishedSession(session: ActiveSession): EstablishedSessionContext =
        session.establishedTransportAndEncryption()
            ?: throw CloseProximityDisclosureException.PlatformException(
                reason = "Session has not been established yet",
            )

    private suspend fun sendTerminatingDeviceResponse(
        session: ActiveSession,
        establishedSessionContext: EstablishedSessionContext,
        deviceResponse: List<UByte>,
    ) {
        try {
            establishedSessionContext.transport.sendMessage(
                buildEncryptedDeviceResponse(
                    sessionEncryption = establishedSessionContext.sessionEncryption,
                    deviceResponse = deviceResponse,
                ),
            )
        } catch (exception: Exception) {
            if (isSessionActive(session)) {
                failSession(session = session, exception = exception)
            }
            throw exception.asPlatformError()
        }
    }

    private fun buildEncryptedDeviceResponse(
        sessionEncryption: SessionEncryption,
        deviceResponse: List<UByte>,
    ): ByteArray =
        sessionEncryption.encryptMessage(
            messagePlaintext = deviceResponse.map { it.toByte() }.toByteArray(),
            statusCode = Constants.SESSION_DATA_STATUS_SESSION_TERMINATION,
        )

    private suspend fun closeSessionAfterDeviceResponse(session: ActiveSession) {
        if (isSessionActive(session)) {
            finishSession(
                session = session,
                update = CloseProximityDisclosureUpdate.Closed,
            )
        }
    }

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

        session.cancelReadJobAndJoin()
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

    private fun createReaderSessionContext(
        eDeviceKey: org.multipaz.crypto.EcPrivateKey,
        encodedDeviceEngagement: ByteArray,
        message: ByteArray,
    ): ReaderSessionContext {
        val eReaderKey = SessionEncryption.getEReaderKey(message)
        val encodedSessionTranscript = buildEncodedSessionTranscript(
            encodedDeviceEngagement = encodedDeviceEngagement,
            encodedReaderKey = eReaderKey.encodedCoseKey,
        )
        return ReaderSessionContext(
            sessionEncryption = SessionEncryption(
                role = MdocRole.MDOC,
                eSelfKey = eDeviceKey,
                remotePublicKey = eReaderKey.publicKey,
                encodedSessionTranscript = encodedSessionTranscript,
            ),
            encodedSessionTranscript = encodedSessionTranscript,
        )
    }

    private fun buildEncodedSessionTranscript(
        encodedDeviceEngagement: ByteArray,
        encodedReaderKey: ByteArray,
    ): ByteArray =
        Cbor.encode(
            buildCborArray {
                add(Tagged(Tagged.ENCODED_CBOR, Bstr(encodedDeviceEngagement)))
                add(Tagged(Tagged.ENCODED_CBOR, Bstr(encodedReaderKey)))
                add(Simple.NULL)
            },
        )

    private suspend fun observeTransportState(
        session: ActiveSession,
        transport: MdocTransport,
    ) {
        try {
            transport.state.collect { state ->
                if (!isSessionActive(session)) {
                    throw StopObservingTransportState()
                }
                handleTransportStateUpdate(session = session, state = state)
            }
        } catch (_: StopObservingTransportState) {
            // Expected when the session is replaced, stopped, or a transport failure is reported elsewhere.
        } catch (exception: Exception) {
            failSession(session = session, exception = exception)
        }
    }

    private suspend fun handleTransportStateUpdate(
        session: ActiveSession,
        state: MdocTransport.State,
    ) {
        when (state) {
            MdocTransport.State.CONNECTED -> {
                session.channel.sendUpdate(update = CloseProximityDisclosureUpdate.Connected)
            }
            MdocTransport.State.CLOSED -> {
                finishSession(session = session, update = CloseProximityDisclosureUpdate.Closed)
                throw StopObservingTransportState()
            }
            MdocTransport.State.FAILED -> {
                // Transport failures are expected to surface through the active connect/read/write
                // operation as well, and that path is responsible for failSession().
                throw StopObservingTransportState()
            }
            else -> Unit
        }
    }

    private suspend fun receiveMessages(
        session: ActiveSession,
        transport: MdocTransport,
    ) {
        var readerSessionContext = session.sessionState().asReaderSessionContext()

        while (isSessionActive(session)) {
            val message = waitForSessionMessage(session = session, transport = transport) ?: return
            readerSessionContext = ensureReaderSessionContext(
                session = session,
                transport = transport,
                message = message,
                currentContext = readerSessionContext,
            ) ?: return
            if (handleReaderMessage(session, message, readerSessionContext)) {
                return
            }
        }
    }

    private suspend fun waitForSessionMessage(
        session: ActiveSession,
        transport: MdocTransport,
    ): ByteArray? {
        // Multipaz explicitly documents and implements close() from another coroutine as the way to
        // interrupt waitForMessage():
        // https://github.com/openwallet-foundation/multipaz/blob/c89f66131e65b7964cdf0fc2961d17cec2b7d781/multipaz/src/commonMain/kotlin/org/multipaz/mdoc/transport/MdocTransportClosedException.kt#L3-L5
        // https://github.com/openwallet-foundation/multipaz/blob/c89f66131e65b7964cdf0fc2961d17cec2b7d781/multipaz/src/commonMain/kotlin/org/multipaz/mdoc/transport/BleTransportPeripheralMdoc.kt#L143-L160
        val message = transport.waitForMessage()
        if (!isSessionActive(session)) {
            return null
        }
        if (message.isEmpty()) {
            finishSession(session = session, update = CloseProximityDisclosureUpdate.Closed)
            return null
        }
        return message
    }

    private suspend fun ensureReaderSessionContext(
        session: ActiveSession,
        transport: MdocTransport,
        message: ByteArray,
        currentContext: ReaderSessionContext?,
    ): ReaderSessionContext? {
        if (currentContext != null) {
            return currentContext
        }

        val newContext = createReaderSessionContext(
            eDeviceKey = session.eDeviceKey,
            encodedDeviceEngagement = session.encodedDeviceEngagement,
            message = message,
        )
        session.setSessionEncryption(
            sessionEncryption = newContext.sessionEncryption,
            encodedSessionTranscript = newContext.encodedSessionTranscript,
        )

        if (!isSessionActive(session)) {
            transport.close()
            return null
        }
        return newContext
    }

    private suspend fun handleReaderMessage(
        session: ActiveSession,
        message: ByteArray,
        readerSessionContext: ReaderSessionContext,
    ): Boolean {
        val (deviceRequest, status) = readerSessionContext.sessionEncryption.decryptMessage(message)
        requireReaderMessageContent(deviceRequest = deviceRequest, status = status)
        if (deviceRequest != null) {
            sendSessionEstablishedUpdate(
                session = session,
                encodedSessionTranscript = readerSessionContext.encodedSessionTranscript,
                deviceRequest = deviceRequest,
            )
        }
        if (status != null) {
            finishSession(session = session, update = status.asCloseProximityDisclosureUpdate())
            return true
        }
        return false
    }

    private fun requireReaderMessageContent(
        deviceRequest: ByteArray?,
        status: Long?,
    ) {
        if (deviceRequest == null && status == null) {
            throw CloseProximityDisclosureException.PlatformException(
                reason = "Reader message did not contain a device request or status",
            )
        }
    }

    private suspend fun sendSessionEstablishedUpdate(
        session: ActiveSession,
        encodedSessionTranscript: ByteArray,
        deviceRequest: ByteArray,
    ) {
        session.channel.sendUpdate(
            update = CloseProximityDisclosureUpdate.SessionEstablished(
                sessionTranscript = encodedSessionTranscript.toUByteList(),
                deviceRequest = deviceRequest.toUByteList(),
            ),
        )
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

    private suspend fun startReadJob(session: ActiveSession) {
        val transport = session.transport() ?: return
        val readJob = bridgeScope.launch {
            try {
                receiveMessages(
                    session = session,
                    transport = transport,
                )
            } catch (_: CancellationException) {
                // The read job is canceled explicitly before sending the terminating DeviceResponse
                // and while shutting the session down, so there is nothing to report here.
            } catch (exception: Exception) {
                failSession(session = session, exception = exception)
            }
        }
        session.setReadJob(readJob)
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

    private fun SessionState.asReaderSessionContext(): ReaderSessionContext? {
        val currentSessionEncryption = sessionEncryption ?: return null
        val currentTranscript = encodedSessionTranscript ?: return null
        return ReaderSessionContext(
            sessionEncryption = currentSessionEncryption,
            encodedSessionTranscript = currentTranscript,
        )
    }

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
