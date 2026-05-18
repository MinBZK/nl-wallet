package helper

import com.codeborne.selenide.Configuration
import data.TestConfigRepository.Companion.testConfig
import driver.BrowserStackMobileDriver
import org.junit.jupiter.api.extension.BeforeAllCallback
import org.junit.jupiter.api.extension.ExtensionContext
import org.junit.jupiter.api.extension.ExtensionContext.Store.CloseableResource
import service.AppiumServiceProvider

class ServiceHelper : BeforeAllCallback {
    override fun beforeAll(context: ExtensionContext) {
        // Start Appium service if running locally
        if (testConfig.remote) {
            Configuration.browser = BrowserStackMobileDriver::class.java.name
        } else {
            context.root.getStore(ExtensionContext.Namespace.GLOBAL)
                .getOrComputeIfAbsent(javaClass) {
                    AppiumServiceProvider.startService()
                    CloseableResource {
                        AppiumServiceProvider.stopService()
                    }
                }
        }

        Configuration.browserSize = null
    }
}
