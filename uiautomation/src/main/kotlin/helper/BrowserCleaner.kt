package helper

import com.codeborne.selenide.WebDriverRunner
import data.TestConfigRepository.Companion.testConfig
import domain.Platform
import io.appium.java_client.AppiumBy
import io.appium.java_client.AppiumDriver
import io.appium.java_client.android.AndroidDriver
import io.appium.java_client.ios.IOSDriver
import org.openqa.selenium.JavascriptExecutor
import org.openqa.selenium.NoSuchElementException
import util.MobileActions
import util.MobileActions.Companion.SCREEN_TRANSITION_MILLIS
import util.MobileActions.Companion.SET_FRAME_SYNC_MAX_WAIT_MILLIS

internal fun clearBrowser(driver: AppiumDriver) {
    if (testConfig.remote) return
    try {
        val platform = driver.capabilities.platformName?.name?.let(Platform::fromString) ?: return
        when (platform) {
            Platform.ANDROID -> clearAndroidBrowser(driver as AndroidDriver)
            Platform.IOS -> clearIosSafariBrowserData(driver as IOSDriver)
        }
    } catch (_: Exception) {}
}

private fun clearAndroidBrowser(driver: AndroidDriver) {
    WebDriverRunner.setWebDriver(driver)
    MobileActions().switchToBrowser()
    Thread.sleep(SCREEN_TRANSITION_MILLIS )
    val webContext = driver.contextHandles.firstOrNull { it.startsWith("WEBVIEW_") } ?: return
    driver.context(webContext)
    driver.switchTo().window(driver.windowHandles.last())
    driver.windowHandles.toList().forEach { handle ->
        driver.switchTo().window(handle)
        try { driver.close() } catch (_: Exception) {}
    }
    try { driver.terminateApp("com.android.chrome") } catch (_: Exception) {}
}

private fun clearIosSafariBrowserData(driver: IOSDriver) {
    WebDriverRunner.setWebDriver(driver)
    driver.activateApp("com.apple.Preferences")
    Thread.sleep(SET_FRAME_SYNC_MAX_WAIT_MILLIS)
    scrollToAndTap(driver, "name == 'Apps'")
    scrollToAndTap(driver, "name == 'Safari'")
    scrollToAndTap(driver, "name == 'CLEAR_HISTORY_AND_DATA'")
    driver.findElements(AppiumBy.iOSNsPredicateString("name == 'CloseAllTabsSwitch'"))
        .firstOrNull { it.isDisplayed && it.getAttribute("value") == "0" }
        ?.click()
    scrollToAndTap(driver, "name == 'ClearHistoryButton'")
    driver.terminateApp("com.apple.Preferences")
    driver.terminateApp("com.apple.mobilesafari")
}

private fun scrollToAndTap(driver: IOSDriver, predicate: String) {
    repeat(8) {
        val matches = driver.findElements(AppiumBy.iOSNsPredicateString(predicate))
        matches.firstOrNull { it.isDisplayed }?.apply {
            click()
            return
        }
        (driver as JavascriptExecutor).executeScript(
            "mobile: scroll",
            mapOf("direction" to "down")
        )
    }
    throw NoSuchElementException("Element not found after scrolling: $predicate")
}
