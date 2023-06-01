package util

import com.codeborne.selenide.WebDriverRunner.getWebDriver
import io.github.ashwith.flutter.FlutterElement
import org.openqa.selenium.remote.RemoteWebDriver

open class MobileActions {

    private val driver = getWebDriver() as RemoteWebDriver
    private val waitMax10Second = 10000
    private val waitMax5Second = 5000
    fun isVisible(element: FlutterElement): Boolean? {
        val result = driver.executeScript("flutter:waitFor", element, waitMax10Second)
        return result as? Boolean
    }

    fun verifyText(element: FlutterElement): String? {
        isVisible(element)
        return element.text
    }

    open fun waitForFirstFrame() {
        driver.executeScript("flutter:waitForFirstFrame")
    }

    fun tapElement(element: FlutterElement) {
        driver.executeScript("flutter:setFrameSync", true, waitMax5Second)
        isVisible(element)
        element.click()
        driver.executeScript("flutter:setFrameSync", false, waitMax5Second)
    }
}
