package util

import com.codeborne.selenide.WebDriverRunner.getWebDriver
import io.github.ashwith.flutter.FlutterElement
import org.openqa.selenium.remote.RemoteWebDriver

open class MobileActions {

    private val driver = getWebDriver() as RemoteWebDriver

    //TODO: Remove hardcoded return boolean (needs assertions refactor/rethinking)
    fun waitForVisibility(element: FlutterElement): Boolean {
        driver.executeScript("flutter:waitFor", element, WAIT_FOR_ELEMENT_MAX_WAIT_MILLIS)
        return true
    }

    fun tapElement(element: FlutterElement) {
        driver.executeScript("flutter:setFrameSync", true, SET_FRAME_SYNC_MAX_WAIT_MILLIS)
        waitForVisibility(element)
        element.click()
        driver.executeScript("flutter:setFrameSync", false, SET_FRAME_SYNC_MAX_WAIT_MILLIS)
    }

    fun readText(element: FlutterElement): String? {
        waitForVisibility(element)
        return element.text
    }

    companion object {
        private const val WAIT_FOR_ELEMENT_MAX_WAIT_MILLIS = 10000
        private const val SET_FRAME_SYNC_MAX_WAIT_MILLIS = 5000
    }
}
