package helper

import com.codeborne.selenide.WebDriverRunner
import data.TestConfigRepository.Companion.testConfig
import domain.Platform
import io.appium.java_client.AppiumBy
import io.appium.java_client.AppiumDriver
import io.appium.java_client.android.AndroidDriver
import io.appium.java_client.ios.IOSDriver
import org.openqa.selenium.JavascriptExecutor
import org.openqa.selenium.remote.RemoteWebElement
import org.openqa.selenium.support.ui.ExpectedConditions
import org.openqa.selenium.support.ui.WebDriverWait
import util.MobileActions
import util.MobileActions.Companion.NATIVE_APP_CONTEXT
import util.MobileActions.Companion.SCREEN_TRANSITION_MILLIS
import util.MobileActions.Companion.SET_FRAME_SYNC_MAX_WAIT_MILLIS
import util.MobileActions.Companion.WAIT_FOR_ELEMENT_MAX_WAIT_MILLIS
import java.time.Duration

private const val SAFARI_BUNDLE_ID = "com.apple.mobilesafari"

internal fun clearBrowser(driver: AppiumDriver) {
    if (testConfig.remote) return
    try {
        val platform = driver.capabilities.platformName?.name?.let(Platform::fromString) ?: return
        when (platform) {
            Platform.ANDROID -> clearAndroidBrowser(driver as AndroidDriver)
            Platform.IOS -> closeAllIosSafariTabs(driver as IOSDriver)
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

private fun closeAllIosSafariTabs(driver: IOSDriver) {
    WebDriverRunner.setWebDriver(driver)
    if (driver.context != NATIVE_APP_CONTEXT) driver.context(NATIVE_APP_CONTEXT)
    try { driver.terminateApp(SAFARI_BUNDLE_ID) } catch (_: Exception) {}
    driver.activateApp(SAFARI_BUNDLE_ID)
    Thread.sleep(SET_FRAME_SYNC_MAX_WAIT_MILLIS)
    val wait = WebDriverWait(driver, Duration.ofMillis(WAIT_FOR_ELEMENT_MAX_WAIT_MILLIS))
    val tabsButton = wait.until(ExpectedConditions.visibilityOfElementLocated(
        AppiumBy.iOSNsPredicateString("type == 'XCUIElementTypeButton' AND name == 'TabOverviewButton'")))
    (driver as JavascriptExecutor).executeScript(
        "mobile: touchAndHold",
        mapOf("elementId" to (tabsButton as RemoteWebElement).id, "duration" to 1.5),
    )

    val closeAllLocator =
        AppiumBy.iOSNsPredicateString("label BEGINSWITH 'Sluit alle' OR label BEGINSWITH 'Close All'")
    val closeBothLocator =
        AppiumBy.iOSNsPredicateString("label BEGINSWITH 'Sluit beide' OR label BEGINSWITH 'Close Both'")
    val closeThisLocator =
        AppiumBy.iOSNsPredicateString("label BEGINSWITH 'Sluit dit' OR label BEGINSWITH 'Close This'")

    wait.until(ExpectedConditions.visibilityOfElementLocated(closeThisLocator))
    val closeButton = driver.findElements(closeAllLocator).firstOrNull()
        ?: driver.findElements(closeBothLocator).firstOrNull()
        ?: driver.findElement(closeThisLocator)
    closeButton.click()
    try { driver.terminateApp(SAFARI_BUNDLE_ID) } catch (_: Exception) {}
}
