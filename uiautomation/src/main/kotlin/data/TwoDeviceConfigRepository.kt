package data

import domain.DeviceCapabilities
import domain.Platform
import domain.TwoDeviceConfig

class TwoDeviceConfigRepository {

    companion object {
        private val appIdentifier = System.getProperty("test.config.app.identifier")
        private val commitSha = System.getProperty("test.config.commit.sha", "")

        private fun buildAppName(platform: Platform): String = when (platform) {
            Platform.ANDROID -> "NLWalletAndroid_${appIdentifier}_$commitSha"
            Platform.IOS -> "NLWalletIos_${appIdentifier}_$commitSha"
        }

        val twoDeviceConfig: TwoDeviceConfig by lazy {
            val sourcePlatform = Platform.fromString(requireNotNull(System.getProperty("test.config.source.platform.name")) {
                "test.config.source.platform.name system property is required for two-device tests"
            })
            val destinationPlatform = Platform.fromString(requireNotNull(System.getProperty("test.config.destination.platform.name")) {
                "test.config.destination.platform.name system property is required for two-device tests"
            })

            TwoDeviceConfig(
                source = DeviceCapabilities(
                    deviceName = System.getProperty("test.config.source.name", ""),
                    platform = sourcePlatform,
                    platformVersion = System.getProperty("test.config.source.platform.version", ""),
                    appName = buildAppName(sourcePlatform),
                    udid = System.getProperty("test.config.source.udid", ""),
                ),
                destination = DeviceCapabilities(
                    deviceName = System.getProperty("test.config.destination.name", ""),
                    platform = destinationPlatform,
                    platformVersion = System.getProperty("test.config.destination.platform.version", ""),
                    appName = buildAppName(destinationPlatform),
                    udid = System.getProperty("test.config.destination.udid", ""),
                ),
            )
        }
    }
}
