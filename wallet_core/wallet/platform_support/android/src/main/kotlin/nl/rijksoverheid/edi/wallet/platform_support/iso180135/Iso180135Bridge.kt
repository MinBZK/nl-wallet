package nl.rijksoverheid.edi.wallet.platform_support.iso180135

import android.content.Context
import nl.rijksoverheid.edi.wallet.platform_support.PlatformSupportInitializer
import uniffi.platform_support.Iso180135channel
import uniffi.platform_support.Iso180135update
import uniffi.platform_support.Iso180135bridge as RustIso180135Bridge

/**
 * This class is automatically initialized on app start through
 * the [PlatformSupportInitializer] class.
 */
class Iso180135Bridge(val context: Context) : RustIso180135Bridge {
    override suspend fun startQrHandover(channel: Iso180135channel): kotlin.String {
        channel.sendUpdate(update = Iso180135update.Connecting)
        Thread.sleep(1_000)

        channel.sendUpdate(update = Iso180135update.Connected)
        Thread.sleep(1_000)

        channel.sendUpdate(update = Iso180135update.DeviceRequest(sessionTranscript = emptyList(), deviceRequest = emptyList()))
        Thread.sleep(1_000)

        channel.sendUpdate(update = Iso180135update.Closed)

        return "some_qr_code"
    }

    override suspend fun sendDeviceResponse(deviceResponse: List<kotlin.UByte>) {}

    override suspend fun stopBleServer() {}
}
