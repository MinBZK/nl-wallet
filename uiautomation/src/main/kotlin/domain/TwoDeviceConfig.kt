package domain

data class TwoDeviceConfig(
    val source: DeviceCapabilities,
    val destination: DeviceCapabilities,
)

data class DeviceCapabilities(
    val deviceName: String,
    val platformName: String,
    val platformVersion: String,
    val appName: String,
    val udid: String = "",
)
