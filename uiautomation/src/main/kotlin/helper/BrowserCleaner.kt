package helper

import com.codeborne.selenide.WebDriverRunner
import data.TestConfigRepository.Companion.testConfig
import io.appium.java_client.AppiumBy
import io.appium.java_client.AppiumDriver
import io.appium.java_client.android.AndroidDriver
import io.appium.java_client.ios.IOSDriver
import org.openqa.selenium.JavascriptExecutor
import org.openqa.selenium.NoSuchElementException
import util.MobileActions

internal fun clearBrowser(driver: AppiumDriver) {
    if (testConfig.remote) return
    try {
        when (driver.capabilities.platformName?.name?.uppercase()) {
            "ANDROID" -> clearAndroidBrowser(driver as AndroidDriver)
            "IOS" -> clearIosSafariBrowserData(driver as IOSDriver)
        }
    } catch (_: Exception) {}
}

private fun clearAndroidBrowser(driver: AndroidDriver) {
    WebDriverRunner.setWebDriver(driver)
    MobileActions().switchToBrowser()
    Thread.sleep(1000)
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
    Thread.sleep(2000)
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
