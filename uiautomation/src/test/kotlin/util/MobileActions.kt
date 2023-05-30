package util

import com.codeborne.selenide.WebDriverRunner.getWebDriver
import io.github.ashwith.flutter.FlutterElement
import org.openqa.selenium.remote.RemoteWebDriver

open class MobileActions {

    private val driver = getWebDriver() as RemoteWebDriver

    fun isVisible(element: FlutterElement): Boolean? {
        val result = driver.executeScript("flutter:waitFor", element)
        return result as? Boolean
    }

    open fun waitForFirstFrame() {
        driver.executeScript("flutter:waitForFirstFrame")
    }

    fun tapElement(element: FlutterElement) {
        driver.executeScript("flutter:setFrameSync", true, 5000)
        element.click()
        driver.executeScript("flutter:setFrameSync", false, 5000)
    }
}
