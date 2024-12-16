package helper

import com.codeborne.selenide.Configuration
import data.TestConfigRepository.Companion.testConfig
import driver.BrowserStackMobileDriver
import driver.LocalMobileDriver
import org.junit.jupiter.api.extension.AfterAllCallback
import org.junit.jupiter.api.extension.BeforeAllCallback
import org.junit.jupiter.api.extension.ExtensionContext
import org.junit.jupiter.api.extension.ExtensionContext.Store.CloseableResource
import service.AppiumServiceProvider

class ServiceHelper : BeforeAllCallback {
    override fun beforeAll(context: ExtensionContext) {
        // Start Appium service if running locally
        if (!testConfig.remote) {
            context.root.getStore(ExtensionContext.Namespace.GLOBAL)
                .getOrComputeIfAbsent(javaClass) {
                    AppiumServiceProvider.startService()
                    CloseableResource {
                        AppiumServiceProvider.stopService()
                    }
             }
        }

        Configuration.browser = if (testConfig.remote) {
            BrowserStackMobileDriver::class.java.name
        } else {
            LocalMobileDriver::class.java.name
        }

        Configuration.browserSize = null
    }
}
