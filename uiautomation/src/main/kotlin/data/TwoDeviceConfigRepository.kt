package data

import domain.DeviceCapabilities
import domain.TwoDeviceConfig

class TwoDeviceConfigRepository {

    companion object {
        private val appIdentifier = System.getProperty("test.config.app.identifier")
        private val commitSha = System.getProperty("test.config.commit.sha", "")

        private fun buildAppName(platformName: String): String = when (platformName) {
            "android" -> "NLWalletAndroid_${appIdentifier}_$commitSha"
            "ios" -> "NLWalletIos_${appIdentifier}_$commitSha"
            else -> throw IllegalArgumentException("Unsupported platform: $platformName")
        }

        val twoDeviceConfig: TwoDeviceConfig by lazy {
            val sourcePlatform = requireNotNull(System.getProperty("test.config.source.platform.name")) {
                "test.config.source.platform.name system property is required for two-device tests"
            }.lowercase()
            val destinationPlatform = requireNotNull(System.getProperty("test.config.destination.platform.name")) {
                "test.config.destination.platform.name system property is required for two-device tests"
            }.lowercase()

            TwoDeviceConfig(
                source = DeviceCapabilities(
                    deviceName = System.getProperty("test.config.source.name", ""),
                    platformName = sourcePlatform,
                    platformVersion = System.getProperty("test.config.source.platform.version", ""),
                    appName = buildAppName(sourcePlatform),
                    udid = System.getProperty("test.config.source.udid", ""),
                ),
                destination = DeviceCapabilities(
                    deviceName = System.getProperty("test.config.destination.name", ""),
                    platformName = destinationPlatform,
                    platformVersion = System.getProperty("test.config.destination.platform.version", ""),
                    appName = buildAppName(destinationPlatform),
                    udid = System.getProperty("test.config.destination.udid", ""),
                ),
            )
        }
    }
}
