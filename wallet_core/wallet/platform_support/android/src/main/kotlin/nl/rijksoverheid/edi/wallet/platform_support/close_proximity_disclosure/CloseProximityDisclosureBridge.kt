package nl.rijksoverheid.edi.wallet.platform_support.close_proximity_disclosure

import android.content.Context
import nl.rijksoverheid.edi.wallet.platform_support.PlatformSupportInitializer
import uniffi.platform_support.CloseProximityDisclosureChannel
import uniffi.platform_support.CloseProximityDisclosureUpdate
import uniffi.platform_support.CloseProximityDisclosureBridge as RustCloseProximityDisclosureBridge

/**
 * This class is automatically initialized on app start through
 * the [PlatformSupportInitializer] class.
 */
class CloseProximityDisclosureBridge(val context: Context) : RustCloseProximityDisclosureBridge {
    override suspend fun startQrHandover(channel: CloseProximityDisclosureChannel): kotlin.String {
        channel.sendUpdate(update = CloseProximityDisclosureUpdate.Connecting)

        channel.sendUpdate(update = CloseProximityDisclosureUpdate.Connected)

        channel.sendUpdate(
            update = CloseProximityDisclosureUpdate.SessionEstablished(
                sessionTranscript = listOf(
                    0x01.toUByte(),
                    0x02.toUByte(),
                    0x03.toUByte()
                ), deviceRequest = listOf(0x04.toUByte(), 0x05.toUByte(), 0x06.toUByte())
            )
        )

        channel.sendUpdate(update = CloseProximityDisclosureUpdate.Closed)

        return "some_qr_code"
    }

    override suspend fun sendDeviceResponse(deviceResponse: List<kotlin.UByte>) {}

    override suspend fun stopBleServer() {}
}
