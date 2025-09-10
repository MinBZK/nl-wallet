package util

import com.codeborne.selenide.WebDriverRunner.getWebDriver
import data.TestConfigRepository.Companion.testConfig
import helper.BrowserStackHelper
import helper.LocalizationHelper
import helper.OrganizationAuthMetadataHelper
import helper.TasDataHelper
import io.appium.java_client.AppiumBy
import io.appium.java_client.AppiumDriver
import io.appium.java_client.android.AndroidDriver
import io.appium.java_client.ios.IOSDriver
import org.openqa.selenium.By
import org.openqa.selenium.JavascriptExecutor
import org.openqa.selenium.WebElement
import org.openqa.selenium.interactions.PointerInput
import org.openqa.selenium.interactions.PointerInput.Origin
import org.openqa.selenium.remote.RemoteWebDriver
import org.openqa.selenium.support.ui.ExpectedConditions
import org.openqa.selenium.support.ui.WebDriverWait
import java.io.IOException
import java.time.Duration

open class NativeMobileActions {

    val driver = getWebDriver() as RemoteWebDriver

    protected val l10n = LocalizationHelper()
    protected val cardMetadata = TasDataHelper()
    protected val organizationAuthMetadata = OrganizationAuthMetadataHelper()

    protected fun isWebElementVisible(element: WebElement): Boolean {
        val wait = WebDriverWait(driver, Duration.ofMillis(WAIT_FOR_ELEMENT_MAX_WAIT_MILLIS))
        wait.until(ExpectedConditions.visibilityOf(element))
        return true
    }

    protected fun clickWebElement(element: WebElement) {
        val wait = WebDriverWait(driver, Duration.ofMillis(WAIT_FOR_ELEMENT_MAX_WAIT_MILLIS))
        wait.until(ExpectedConditions.visibilityOf(element))
        element.click()
    }

    protected fun findElement(locator: By): WebElement {
        val wait = WebDriverWait(driver, Duration.ofMillis(WAIT_FOR_ELEMENT_MAX_WAIT_MILLIS))
        wait.until(ExpectedConditions.visibilityOfElementLocated(locator))
        return driver.findElement(locator)
    }

    protected fun getTopLeftOfElementWithText(text: String): Pair<Double, Double>? =
        try {
            val element = findElementByText(text)
            val r = element.rect
            Pair(r.x.toDouble(), r.y.toDouble())
        } catch (_: Exception) {
            null
        }

    fun scrollToElementWithText(text: String): WebElement {
        return when (val platform = platformName()) {
            "ANDROID" -> driver.findElement(
                AppiumBy.androidUIAutomator(
                    "new UiScrollable(new UiSelector().scrollable(true))" +
                        ".scrollIntoView(new UiSelector().description(\"$text\"))"
                )
            )
            "IOS" -> {
                val predicate = "name == '$text'"
                (driver as JavascriptExecutor).executeScript(
                    "mobile: scroll",
                    mapOf("predicateString" to predicate, "toVisible" to true)
                )
                driver.findElement(AppiumBy.iOSNsPredicateString(predicate))
            }
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
    }

    fun scrollToEndOfScreen(durationMs: Int = 300) {
        val driver = when (val platform = platformName()) {
            "ANDROID" -> driver as AndroidDriver
            "IOS" -> driver as IOSDriver
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }

        val size = driver.manage().window().size
        val width = size.width
        val height = size.height

        val startX = width / 2
        val startY = (height * 0.80).toInt()   // near bottom
        val endX   = startX
        val endY   = (height * 0.20).toInt()   // near top

        val finger = PointerInput(PointerInput.Kind.TOUCH, "finger")
        val swipe = org.openqa.selenium.interactions.Sequence(finger, 1)
            .addAction(finger.createPointerMove(Duration.ZERO, Origin.viewport(), startX, startY))
            .addAction(finger.createPointerDown(PointerInput.MouseButton.LEFT.asArg()))
            .addAction(finger.createPointerMove(Duration.ofMillis(durationMs.toLong()), Origin.viewport(), endX, endY))
            .addAction(finger.createPointerUp(PointerInput.MouseButton.LEFT.asArg()))

        driver.perform(listOf(swipe))
    }

    fun switchToWebViewContext() {
        val driver = when (val platform = platformName()) {
            "ANDROID" -> driver as AndroidDriver
            "IOS" -> driver as IOSDriver
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
        val context = driver.context ?: ""
        if (context.startsWith(WEB_VIEW_CONTEXT_PREFIX).not()) {
            // Wait for the web view context to be available
            val wait = WebDriverWait(driver, Duration.ofMillis(WAIT_FOR_CONTEXT_MAX_WAIT_MILLIS))
            val contextHandle = wait.until {
                driver.contextHandles.firstOrNull { it.startsWith(WEB_VIEW_CONTEXT_PREFIX) }
            }

            // Switch to the web view context
            driver.context(contextHandle)

            // Explicit timeout; waiting for the browser to be fully started and the viewport stabilized.
            // This fixes the issue where the (Chrome) browser viewport flickers back and forth between
            // the loaded web page and the browser startup screen shortly after browser startup.
            Thread.sleep(BROWSER_STARTUP_TIMEOUT)

            // Switch to the last window handle (a.k.a. tab) if local
            val windowHandles = if (testConfig.remote) { setOf() } else { (driver as AppiumDriver).windowHandles }
            if (windowHandles.isNotEmpty()) {
                if (driver is IOSDriver) {
                    driver.switchTo().window(windowHandles.last())
                } else {
                    driver.switchTo().window(windowHandles.first())
                }
            }
        }
    }

    fun switchToNativeContext() {
        val driver = when (val platform = platformName()) {
            "ANDROID" -> driver as AndroidDriver
            "IOS" -> driver as IOSDriver
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
        if (driver.context != NATIVE_APP_CONTEXT) {
            driver.context(NATIVE_APP_CONTEXT)
        }
    }

    protected fun getWebModalAnchor(): WebElement {
        Thread.sleep(MODAL_ANCHOR_DISPLAY_TIMEOUT)
        val startButton = driver.findElement(By.tagName("nl-wallet-button"))

        val jsExecutor = driver as JavascriptExecutor
        val jsScript = "return arguments[0].querySelector('.modal-anchor')"
        return jsExecutor.executeScript(jsScript, startButton.shadowRoot) as WebElement
    }

    fun tapCoordinates(x: Int, y: Int) {
        Thread.sleep(PAGE_LOAD_TIMEOUT)
        try {
            val finger = PointerInput(PointerInput.Kind.TOUCH, "finger")

            val tap = org.openqa.selenium.interactions.Sequence(finger, 0)
            tap.addAction(finger.createPointerMove(Duration.ZERO, Origin.viewport(), x, y))
            tap.addAction(finger.createPointerDown(PointerInput.MouseButton.LEFT.asArg()))
            tap.addAction(finger.createPointerUp(PointerInput.MouseButton.LEFT.asArg()))

            driver.perform(listOf(tap))

            println("Tapped at coordinates ($x, $y)")
        } catch (e: Exception) {
            println("Failed to tap at coordinates ($x, $y): ${e.message}")
            throw e
        }
    }

    fun platformName() = driver.capabilities.platformName?.name ?: throw IllegalStateException("No platform name")

    fun disableInternetConnection() {
        if (testConfig.remote) {
            val sessionId = driver.sessionId.toString()
            BrowserStackHelper.setNetwork(BROWSERSTACK_ENDPOINT, browserStackUserName, browserStackAccessKey, sessionId, "no-network" )
        } else {
            when (val platform = platformName()) {
                "ANDROID" -> {
                    try {
                        runCommand(listOf("adb", "shell", "svc", "wifi", "disable"))
                        runCommand(listOf("adb", "shell", "svc", "data", "disable"))
                    } catch (e: IOException) {
                        e.printStackTrace()
                        throw RuntimeException("Failed to disable network via ADB", e)
                    }
                }
                "IOS" -> {
                    throw UnsupportedOperationException("Disabling network not supported on iOS via code. Consider using a manual toggle.")
                }
                else -> {
                    throw IllegalArgumentException("Unsupported platform: $platform")
                }
            }
        }
    }

    fun enableInternetConnection() {
        if (testConfig.remote) {
            val sessionId = driver.sessionId.toString()
            BrowserStackHelper.setNetwork(BROWSERSTACK_ENDPOINT, browserStackUserName, browserStackAccessKey, sessionId, "reset" )
        } else {
            when (val platform = platformName()) {
                "ANDROID" -> {
                    try {
                        runCommand(listOf("adb", "shell", "svc", "wifi", "enable"))
                        runCommand(listOf("adb", "shell", "svc", "data", "enable"))
                    } catch (e: IOException) {
                        e.printStackTrace()
                        throw RuntimeException("Failed to enable network via ADB", e)
                    }
                }
                "IOS" -> {
                    throw UnsupportedOperationException("Re-enabling network not supported on iOS via code.")
                }
                else -> {
                    throw IllegalArgumentException("Unsupported platform: $platform")
                }
            }
        }
    }

    private fun runCommand(command: List<String>) {
        val builder = ProcessBuilder(command)
        val process = builder.start()
        process.waitFor()
    }

    fun getTextFromElementContainingText(partialText: String): String? {
        val element = try {
            findElementByPartialText(partialText)
        } catch (e: Exception) {
            println("Failed to get element text: ${e.message}")
            null
        }

        if (element == null) {
            throw NoSuchElementException("No element found containing: $partialText")
        }

        return when (val platform = platformName()) {
            "ANDROID" -> element.getAttribute("contentDescription")
            "IOS" -> element.getAttribute("name")
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
    }

    fun clickElementContainingText(partialText: String) {
        return try {
            findElementByPartialText(partialText).click()
        } catch (e: Exception) {
            println("Failed to get element: ${e.message}")
        }
    }

    fun clickElementWithText(text: String) {
        return try {
            findElementByText(text).click()
        } catch (e: Exception) {
            println("Failed to get element: ${e.message}")
        }
    }

    fun elementContainingTextVisible(partialText: String): Boolean {
        return try {
            findElementByPartialText(partialText).isDisplayed
        } catch (e: Exception) {
            println("Element not found or error occurred: ${e.message}")
            false
        }
    }

    fun elementWithTextVisible(text: String): Boolean {
        return try {
            findElementByText(text).isDisplayed
        } catch (e: Exception) {
            println("Element not found or error occurred: ${e.message}")
            false
        }
    }

    fun elementWithDescendantAndTextAndVisible(
        descendantElementText: String,
        elementText: String
    ): Boolean {
        return try {
            findElementByDescendantElementText(descendantElementText, elementText).isDisplayed
        } catch (e: Exception) {
            println("Element not found or error occurred: ${e.message}")
            false
        }
    }

    private fun findElementByPartialText(partialText: String, timeoutInSeconds: Long = 5): WebElement {
        val wait = WebDriverWait(driver, Duration.ofSeconds(timeoutInSeconds))
        return when (val platform = platformName()) {
            "ANDROID" -> wait.until(
                ExpectedConditions.presenceOfElementLocated(
                    AppiumBy.androidUIAutomator("new UiSelector().descriptionContains(\"$partialText\")")
                )
            )
            "IOS" -> wait.until(
                ExpectedConditions.presenceOfElementLocated(
                    By.xpath("//*[contains(@name, '$partialText')]")
                )
            )
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
    }

    private fun findElementByText(text: String, timeoutInSeconds: Long = 5): WebElement {
        val wait = WebDriverWait(driver, Duration.ofSeconds(timeoutInSeconds))
        return when (val platform = platformName()) {
            "ANDROID" -> wait.until(
                ExpectedConditions.presenceOfElementLocated(
                    AppiumBy.androidUIAutomator("new UiSelector().description(\"$text\")")
                )
            )
            "IOS" -> wait.until(
                ExpectedConditions.presenceOfElementLocated(
                    AppiumBy.iOSNsPredicateString("name == '$text'")
                )
            )
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
    }

    private fun findElementByDescendantElementText(
        descendantElementText: String,
        elementText: String
    ): WebElement {
        return when (val platform = platformName()) {
            "ANDROID" -> driver.findElement(
                By.xpath("//*[@content-desc='$elementText' and .//*[@content-desc='$descendantElementText']]")
            )
            "IOS" -> driver.findElement(
                By.xpath("//*[@name='$elementText' and .//*[@name='$descendantElementText']]")
            )
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
    }

    fun switchToWalletApp() {
        val driver = when (val platform = platformName()) {
            "ANDROID" -> driver as AndroidDriver
            "IOS" -> driver as IOSDriver
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
        driver.activateApp(testConfig.appIdentifier)
    }

    fun switchToBrowser() {
        val driver = when (val platform = platformName()) {
            "ANDROID" -> driver as AndroidDriver
            "IOS" -> driver as IOSDriver
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
        when (platformName()) {
            "ANDROID" -> driver.activateApp("com.android.chrome")
            "IOS" -> driver.activateApp("com.apple.mobilesafari")
        }
    }

    fun printPageSource() {
        val driver = driver as AppiumDriver
        println(driver.pageSource)
    }

    fun putAppInBackground(seconds: Int) {
        val driver = when (val platform = platformName()) {
            "ANDROID" -> driver as AndroidDriver
            "IOS" -> driver as IOSDriver
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
        driver.runAppInBackground(Duration.ofSeconds(seconds.toLong()))
    }

    fun openUniversalLink(expiredUniversalLinkFromCameraApp: String) {
        val driver = driver as AppiumDriver
        driver.get(expiredUniversalLinkFromCameraApp)
        Thread.sleep(SET_FRAME_SYNC_MAX_WAIT_MILLIS)
    }

    companion object {
        private const val SET_FRAME_SYNC_MAX_WAIT_MILLIS = 2000L
        private const val WAIT_FOR_ELEMENT_MAX_WAIT_MILLIS = 8000L
        private const val WAIT_FOR_CONTEXT_MAX_WAIT_MILLIS = 4000L
        private const val BROWSER_STARTUP_TIMEOUT = 2000L
        const val PAGE_LOAD_TIMEOUT = 4000L
        private const val MODAL_ANCHOR_DISPLAY_TIMEOUT = 500L

        private const val WEB_VIEW_CONTEXT_PREFIX = "WEBVIEW_"
        private const val NATIVE_APP_CONTEXT = "NATIVE_APP"

        private val browserStackUserName = EnvironmentUtil.getVar("BROWSERSTACK_USER")
        private val browserStackAccessKey = EnvironmentUtil.getVar("BROWSERSTACK_KEY")
        private const val BROWSERSTACK_ENDPOINT = "https://api.browserstack.com/app-automate/sessions/"
    }
}
