package data

import domain.Platform
import domain.TestConfig

class TestConfigRepository {

    companion object {
        val testConfig = TestConfig(
            appIdentifier = System.getProperty("test.config.app.identifier"),
            deviceName = System.getProperty("test.config.device.name"),
            platform = Platform.fromString(System.getProperty("test.config.platform.name")),
            platformVersion = System.getProperty("test.config.platform.version"),
            udid = System.getProperty("test.config.device.udid", ""),
            remote = System.getProperty("test.config.remote").toBoolean(),
            automationName = System.getProperty("test.config.automation.name"),
            commitSha = System.getProperty("test.config.commit.sha"),
        )
    }
}
