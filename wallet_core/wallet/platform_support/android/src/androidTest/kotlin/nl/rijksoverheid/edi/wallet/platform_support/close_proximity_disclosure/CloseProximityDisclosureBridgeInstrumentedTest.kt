package nl.rijksoverheid.edi.wallet.platform_support.close_proximity_disclosure

import android.Manifest
import android.bluetooth.BluetoothManager
import android.content.pm.PackageManager
import android.util.Log
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.filters.RequiresDevice
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.rule.GrantPermissionRule
import kotlinx.coroutines.async
import kotlinx.coroutines.asCoroutineDispatcher
import kotlinx.coroutines.test.runTest
import nl.rijksoverheid.edi.wallet.platform_support.PlatformSupport
import org.junit.Assert.assertNull
import org.junit.After
import org.junit.Assume.assumeTrue
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.multipaz.util.Constants
import java.util.concurrent.CopyOnWriteArrayList
import java.util.concurrent.CountDownLatch
import java.util.concurrent.CyclicBarrier
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import uniffi.platform_support.CloseProximityDisclosureChannel
import uniffi.platform_support.CloseProximityDisclosureUpdate
import uniffi.platform_support.NoHandle

@RunWith(AndroidJUnit4::class)
@RequiresDevice
class CloseProximityDisclosureBridgeInstrumentedTest {
    private sealed interface RecordedUpdate {
        data object Connecting : RecordedUpdate
        data class SessionEstablished(
            val sessionTranscript: List<UByte>,
            val deviceRequest: List<UByte>,
        ) : RecordedUpdate
        data object Closed : RecordedUpdate
        data object Other : RecordedUpdate
    }

    @get:Rule
    val grantBluetoothPermissions: GrantPermissionRule = GrantPermissionRule.grant(
        Manifest.permission.BLUETOOTH_ADVERTISE,
        Manifest.permission.BLUETOOTH_CONNECT,
    )

    private class TestChannel : CloseProximityDisclosureChannel(NoHandle) {
        private val updates = CopyOnWriteArrayList<RecordedUpdate>()
        private val connectingLatch = CountDownLatch(1)
        private val sessionEstablishedLatch = CountDownLatch(1)
        private val closedLatch = CountDownLatch(1)

        override suspend fun sendUpdate(update: CloseProximityDisclosureUpdate) {
            when (update) {
                is CloseProximityDisclosureUpdate.Connecting -> {
                    updates.add(RecordedUpdate.Connecting)
                    connectingLatch.countDown()
                }
                is CloseProximityDisclosureUpdate.SessionEstablished -> {
                    updates.add(
                        RecordedUpdate.SessionEstablished(
                            sessionTranscript = update.sessionTranscript,
                            deviceRequest = update.deviceRequest,
                        ),
                    )
                    sessionEstablishedLatch.countDown()
                }
                is CloseProximityDisclosureUpdate.Closed -> {
                    updates.add(RecordedUpdate.Closed)
                    closedLatch.countDown()
                }
                else -> updates.add(RecordedUpdate.Other)
            }
        }

        fun awaitConnectingUpdate(timeoutSeconds: Long = 20): Boolean =
            connectingLatch.await(timeoutSeconds, TimeUnit.SECONDS)

        fun awaitSessionEstablishedUpdate(timeoutSeconds: Long = 30): Boolean =
            sessionEstablishedLatch.await(timeoutSeconds, TimeUnit.SECONDS)

        fun awaitClosedUpdate(timeoutSeconds: Long = 1): Boolean =
            closedLatch.await(timeoutSeconds, TimeUnit.SECONDS)

        fun hasReceivedConnectingUpdate(): Boolean = connectingLatch.count == 0L

        fun hasReceivedSessionEstablishedUpdate(): Boolean = sessionEstablishedLatch.count == 0L

        fun hasReceivedClosedUpdate(): Boolean = closedLatch.count == 0L

        fun receivedSessionEstablishedUpdate(): RecordedUpdate.SessionEstablished? =
            updates.firstOrNull { it is RecordedUpdate.SessionEstablished } as? RecordedUpdate.SessionEstablished

        fun receivedUpdates(): List<RecordedUpdate> = updates.toList()
    }

    private lateinit var closeProximityDisclosureBridge: CloseProximityDisclosureBridge

    @Before
    fun setUp() = runTest {
        val context = InstrumentationRegistry.getInstrumentation().context
        val packageManager = context.packageManager
        assertTrue(
            "BLE is not supported on this device",
            packageManager.hasSystemFeature(PackageManager.FEATURE_BLUETOOTH_LE),
        )

        val bluetoothManager = context.getSystemService(BluetoothManager::class.java)
        assertNotNull("BluetoothManager is unavailable", bluetoothManager)

        val bluetoothAdapter = bluetoothManager?.adapter
        assertNotNull("Bluetooth adapter is unavailable", bluetoothAdapter)
        assertTrue(
            "Bluetooth must be enabled before running close proximity disclosure tests",
            bluetoothAdapter?.isEnabled == true,
        )
        assertNotNull(
            "BLE advertising is not supported on this device",
            bluetoothAdapter?.bluetoothLeAdvertiser,
        )

        closeProximityDisclosureBridge = PlatformSupport.getInstance(context).closeProximityDisclosureBridge
        closeProximityDisclosureBridge.stopBleServer()
    }

    @After
    fun tearDown() = runTest {
        closeProximityDisclosureBridge.stopBleServer()
    }

    @Test
    fun bridge_test_start_qr_handover() {
        // Explicitly load platform_support since close_proximity_disclosure_test_start_qr_handover() is stripped from rust_core
        System.loadLibrary("platform_support")

        // The Rust code will panic if this test fails.
        close_proximity_disclosure_test_start_qr_handover()
    }

    @Test
    fun test_start_qr_handover_starts_and_stops_ble_server() = runTest {
        val channel = TestChannel()

        assertFalse(closeProximityDisclosureBridge.isBleServerActiveForTesting())

        val qrCode = closeProximityDisclosureBridge.startQrHandover(channel)

        Log.i(CLOSE_PROXIMITY_TEST_LOG_TAG, "Close proximity disclosure QR code: $qrCode")

        assertTrue(qrCode.isNotEmpty())
        assertFalse(qrCode.startsWith("mdoc:"))
        assertTrue(closeProximityDisclosureBridge.isBleServerActiveForTesting())

        closeProximityDisclosureBridge.stopBleServer()

        assertTrue(channel.hasReceivedClosedUpdate())
        assertFalse(closeProximityDisclosureBridge.isBleServerActiveForTesting())
    }

    @Test
    fun test_session_establishment_failure_status_maps_cbor_errors_to_11() {
        assertEquals(
            Constants.SESSION_DATA_STATUS_ERROR_CBOR_DECODING,
            sessionEstablishmentFailureStatus(IllegalArgumentException("bad cbor")),
        )
        assertEquals(
            Constants.SESSION_DATA_STATUS_ERROR_CBOR_DECODING,
            sessionEstablishmentFailureStatus(IllegalStateException("missing eReaderKey")),
        )
    }

    @Test
    fun test_session_establishment_failure_status_ignores_other_errors() {
        assertNull(sessionEstablishmentFailureStatus(RuntimeException("boom")))
    }

    @Test
    fun test_session_message_failure_status_maps_cbor_and_decryption_errors() {
        assertEquals(
            Constants.SESSION_DATA_STATUS_ERROR_CBOR_DECODING,
            sessionMessageFailureStatus(IllegalArgumentException("bad session data")),
        )
        assertEquals(
            Constants.SESSION_DATA_STATUS_ERROR_SESSION_ENCRYPTION,
            sessionMessageFailureStatus(IllegalStateException("decryption failed")),
        )
    }

    @Test
    fun test_session_message_failure_status_ignores_other_errors() {
        assertNull(sessionMessageFailureStatus(RuntimeException("boom")))
    }

    @Test
    fun test_close_proximity_disclosure_full_flow_with_mac_reader() = runTest {
        assumeTrue(
            "Set RUN_MAC_BLE_READER_FULL_FLOW_TEST = true and run scripts/close_proximity/disclosure_mac_reader.swift --qr-code <logged-qr-code> --expect-device-response-hex $MAC_BLE_READER_EXPECTED_DEVICE_RESPONSE_HEX on the host Mac to exercise the full flow",
            RUN_MAC_BLE_READER_FULL_FLOW_TEST,
        )

        val channel = TestChannel()

        try {
            closeProximityDisclosureBridge.stopBleServer()

            assertFalse(closeProximityDisclosureBridge.isBleServerActiveForTesting())

            val qrCode = closeProximityDisclosureBridge.startQrHandover(channel)

            Log.i(CLOSE_PROXIMITY_TEST_LOG_TAG, "Close proximity disclosure QR code: $qrCode")
            Log.i(
                CLOSE_PROXIMITY_TEST_LOG_TAG,
                macBleReaderMarkerPayload(
                    qrCode = qrCode,
                    expectedDeviceResponseHex = MAC_BLE_READER_EXPECTED_DEVICE_RESPONSE_HEX,
                ),
            )

            assertTrue(qrCode.isNotEmpty())
            assertFalse(qrCode.startsWith("mdoc:"))
            assertTrue(closeProximityDisclosureBridge.isBleServerActiveForTesting())

            assertTrue(
                "Timed out waiting for the host Mac BLE helper to send SessionEstablished",
                channel.awaitSessionEstablishedUpdate(),
            )
            assertTrue(channel.hasReceivedSessionEstablishedUpdate())

            val sessionEstablished = channel.receivedSessionEstablishedUpdate()
            assertNotNull("Expected a SessionEstablished update", sessionEstablished)
            assertEquals(
                MAC_BLE_READER_EXPECTED_DEVICE_REQUEST,
                sessionEstablished?.deviceRequest,
            )

            closeProximityDisclosureBridge.sendDeviceResponse(MAC_BLE_READER_EXPECTED_DEVICE_RESPONSE)

            assertTrue(
                "Timed out waiting for the wallet to close the BLE session after sending the encrypted DeviceResponse. The host Mac BLE helper validates the DeviceResponse out of process and will fail the overall run separately if validation fails.",
                channel.awaitClosedUpdate(timeoutSeconds = 5),
            )
            assertTrue(channel.hasReceivedClosedUpdate())

            val updatesAfterClose = channel.receivedUpdates()
            val connectingIndex = updatesAfterClose.indexOf(RecordedUpdate.Connecting)
            val sessionEstablishedIndex = updatesAfterClose.indexOfFirst {
                it is RecordedUpdate.SessionEstablished
            }
            val closedIndex = updatesAfterClose.indexOf(RecordedUpdate.Closed)
            assertTrue(connectingIndex >= 0)
            assertTrue(sessionEstablishedIndex >= 0)
            assertTrue(closedIndex >= 0)
            assertTrue(connectingIndex < sessionEstablishedIndex)
            assertTrue(sessionEstablishedIndex < closedIndex)
            assertFalse(closeProximityDisclosureBridge.isBleServerActiveForTesting())
        } finally {
            closeProximityDisclosureBridge.stopBleServer()
        }
    }

    @Test
    fun test_start_qr_handover_from_two_threads_replaces_previous_session() = runTest {
        val firstChannel = TestChannel()
        val secondChannel = TestChannel()

        Executors.newFixedThreadPool(2).asCoroutineDispatcher().use { dispatcher ->
            val barrier = CyclicBarrier(2)
            val firstCall = async(dispatcher) {
                barrier.await()
                closeProximityDisclosureBridge.startQrHandover(firstChannel)
            }
            val secondCall = async(dispatcher) {
                barrier.await()
                closeProximityDisclosureBridge.startQrHandover(secondChannel)
            }

            val firstQrCode = firstCall.await()
            val secondQrCode = secondCall.await()

            assertTrue(firstQrCode.isNotEmpty())
            assertTrue(secondQrCode.isNotEmpty())
            assertFalse(firstQrCode.startsWith("mdoc:"))
            assertFalse(secondQrCode.startsWith("mdoc:"))
        }

        assertTrue(closeProximityDisclosureBridge.isBleServerActiveForTesting())

        val firstClosed = firstChannel.awaitClosedUpdate()
        val secondClosed = secondChannel.awaitClosedUpdate()

        assertTrue(firstClosed.xor(secondClosed))

        val replacedChannel = if (firstClosed) firstChannel else secondChannel
        val activeChannel = if (firstClosed) secondChannel else firstChannel

        val replacedUpdatesBeforeStop = replacedChannel.receivedUpdates()
        val activeUpdatesBeforeStop = activeChannel.receivedUpdates()
        assertEquals(
            listOf(
                RecordedUpdate.Closed,
            ),
            replacedUpdatesBeforeStop,
        )

        assertTrue(activeUpdatesBeforeStop.isEmpty())

        closeProximityDisclosureBridge.stopBleServer()

        assertTrue(activeChannel.awaitClosedUpdate())
        val activeUpdatesAfterStop = activeChannel.receivedUpdates()
        val replacedUpdatesAfterStop = replacedChannel.receivedUpdates()
        assertEquals(
            listOf(
                RecordedUpdate.Closed,
            ),
            activeUpdatesAfterStop,
        )
        assertEquals(replacedUpdatesBeforeStop, replacedUpdatesAfterStop)
        assertFalse(closeProximityDisclosureBridge.isBleServerActiveForTesting())
    }

    companion object {
        private const val CLOSE_PROXIMITY_TEST_LOG_TAG = "CloseProximityTest"
        private const val RUN_MAC_BLE_READER_FULL_FLOW_TEST = true
        private const val MAC_BLE_READER_MARKER = "CLOSE_PROXIMITY_MAC_READER="
        private const val MAC_BLE_READER_EXPECTED_DEVICE_RESPONSE_HEX = "040506"
        private val MAC_BLE_READER_EXPECTED_DEVICE_REQUEST =
            listOf(0x01u.toUByte(), 0x02u.toUByte(), 0x03u.toUByte())
        private val MAC_BLE_READER_EXPECTED_DEVICE_RESPONSE =
            listOf(0x04u.toUByte(), 0x05u.toUByte(), 0x06u.toUByte())

        private fun macBleReaderMarkerPayload(
            qrCode: String,
            expectedDeviceResponseHex: String? = null,
        ): String =
            if (expectedDeviceResponseHex != null) {
                "$MAC_BLE_READER_MARKER{\"qr\":\"$qrCode\",\"expected_device_response_hex\":\"$expectedDeviceResponseHex\"}"
            } else {
                "$MAC_BLE_READER_MARKER{\"qr\":\"$qrCode\"}"
            }

        @JvmStatic
        external fun close_proximity_disclosure_test_start_qr_handover()
    }
}
