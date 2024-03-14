package data

import domain.TestConfig

class TestConfigRepository {

    companion object {
        val testConfig = TestConfig(
            appIdentifier = System.getProperty("test.config.app.identifier"),
            deviceName = System.getProperty("test.config.device.name"),
            platformName = System.getProperty("test.config.platform.name").lowercase(),
            platformVersion = System.getProperty("test.config.platform.version"),
            remote = System.getProperty("test.config.remote").toBoolean(),
        )
    }
}
