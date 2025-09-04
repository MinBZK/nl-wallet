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
import io.github.ashwith.flutter.FlutterElement
import io.github.ashwith.flutter.FlutterFinder
import org.openqa.selenium.By
import org.openqa.selenium.InvalidArgumentException
import org.openqa.selenium.JavascriptExecutor
import org.openqa.selenium.TimeoutException
import org.openqa.selenium.WebDriverException
import org.openqa.selenium.WebElement
import org.openqa.selenium.interactions.PointerInput
import org.openqa.selenium.remote.RemoteWebDriver
import org.openqa.selenium.support.ui.ExpectedConditions
import org.openqa.selenium.support.ui.WebDriverWait
import java.io.IOException
import java.time.Duration

open class MobileActions {

    private val driver = getWebDriver() as RemoteWebDriver

    protected val find = FlutterFinder(driver)
    protected val l10n = LocalizationHelper()
    protected val cardMetadata = TasDataHelper()
    protected val organizationAuthMetadata = OrganizationAuthMetadataHelper()

    /**
     * Checks if the given element is visible.
     * This method will wait for the element to be visible for a maximum of 5 seconds.
     *
     * By default, Flutter Driver waits until there is no pending frame scheduled in the app under test,
     * before executing an action or a test assertion.
     *
     * Do not use this method for checking if an element is absent, use the `isElementAbsent` method instead.
     *
     * @param element The element to check for visibility.
     * @param frameSync Whether to wait executing an action until no pending frames are scheduled. Defaults to `true`, set to `false` when testing a screen with a long or infinite animation.
     */
    protected fun isElementVisible(element: FlutterElement, frameSync: Boolean = true): Boolean {
        return try {
            performAction(frameSync) {
                driver.executeScript("flutter:waitFor", element, WAIT_FOR_ELEMENT_MAX_WAIT_MILLIS)
            }
            true
        } catch (e: TimeoutException) {
            println("TimeoutException: Element not visible within ${WAIT_FOR_ELEMENT_MAX_WAIT_MILLIS}ms — $element")
            false
        } catch (e: NoSuchElementException) {
            println("NoSuchElementException: Element not found in the widget tree — $element")
            false
        } catch (e: InvalidArgumentException) {
            println("InvalidArgumentException: Invalid argument passed to driver.executeScript — $element")
            false
        } catch (e: WebDriverException) {
            println("WebDriverException: Communication issue with the driver while checking visibility — $element")
            println("   → ${e.message}")
            false
        }
    }

    protected fun isElementAbsent(element: FlutterElement, frameSync: Boolean = true): Boolean {
        performAction(frameSync) {
            driver.executeScript("flutter:waitForAbsent", element, WAIT_FOR_ELEMENT_MAX_WAIT_MILLIS)
        }

        // Ideally we would use `!element.isDisplayed` as return value,
        // but this isn't possible due to missing `FlutterElement` implementations.
        //
        // The `driver.executeScript()` method will throw (fail test) when the element is not absent,
        // therefore returning hardcode value for the sake of test assertions.
        return true
    }

    protected fun isWebElementVisible(element: WebElement): Boolean {
        val wait = WebDriverWait(driver, Duration.ofMillis(WAIT_FOR_ELEMENT_MAX_WAIT_MILLIS))
        wait.until(ExpectedConditions.visibilityOf(element))
        return true
    }

    protected fun clickWebElement(element: WebElement) {
        // Wait for the element to be visible before clicking it
        val wait = WebDriverWait(driver, Duration.ofMillis(WAIT_FOR_ELEMENT_MAX_WAIT_MILLIS))
        wait.until(ExpectedConditions.visibilityOf(element))
        element.click()
    }

    protected fun findElement(locator: By): WebElement = driver.findElement(locator)

    protected enum class ScrollableType {
        CustomScrollView,
        ListView,
    }

    protected fun scrollToEnd(element: FlutterElement) {
        val args = mapOf("dx" to 0, "dy" to -2000, "durationMilliseconds" to 100, "frequency" to 100)
        driver.executeScript("flutter:scroll", element, args)
    }

    protected fun scrollToEndOnDashBoard() {
        val dashboardFinder = find.byType("DashboardScreen")
        val customScrollFinder = find.byDescendant(
            dashboardFinder,
            find.byType("ListView"),
            true, true
        )
        val args = mapOf(
            "dx" to 0,
            "dy" to -300,
            "durationMilliseconds" to 500
        )
        driver.executeScript("flutter:scroll", customScrollFinder, args)
    }

    protected fun clickElement(element: FlutterElement, frameSync: Boolean = true) {
        // First wait and check if the element is visible, then perform the click action;
        // this prevents clicking on an element that is not visible, which results in a (BrowserStack) timeout.
        if (isElementVisible(element, frameSync)) {
            performAction(frameSync) { element.click() }
        }
    }

    /**
     * Resolves the top left coordinates of the given element in logical pixels (not taking into account the screen density).
     *
     * @param element The element to get the top left coordinates from.
     * @param frameSync Whether to wait executing an action until no pending frames are scheduled. Defaults to `true`, set to `false` when testing a screen with a long or infinite animation.
     * @return The top left coordinates of the given element in logical pixels, or `null` if the element is not found.
     */
    protected fun getTopLeft(element: WebElement, frameSync: Boolean = true): Pair<Double, Double>? {
        return performAction(frameSync) {

            when (val result = driver.executeScript("flutter:getTopLeft", element)) {
                is Map<*, *> -> {
                    val dx = (result["dx"] as? Number)?.toDouble() ?: return@performAction null
                    val dy = (result["dy"] as? Number)?.toDouble() ?: return@performAction null
                    Pair(dx, dy)
                }

                else -> null
            }
        }
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
                    // Wait somewhat more on iOS
                    Thread.sleep(BROWSER_STARTUP_TIMEOUT)
                } else {
                    driver.switchTo().window(windowHandles.first())
                }
            }
        }
    }

    fun switchToAppContext() {
        val driver = when (val platform = platformName()) {
            "ANDROID" -> driver as AndroidDriver
            "IOS" -> driver as IOSDriver
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
        if (driver.context != FLUTTER_APP_CONTEXT) {
            // Switch to the app context
            driver.context(FLUTTER_APP_CONTEXT)
            driver.terminateApp("com.android.chrome")
        }
    }

    private fun switchToNativeContext() {
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
        // Wait for the modal-anchor element to be displayed
        Thread.sleep(MODAL_ANCHOR_DISPLAY_TIMEOUT)

        // Locate shadow-host element
        val startButton = driver.findElement(By.tagName("nl-wallet-button"))

        // Locate modal-anchor element inside the shadow-root of the shadow-host element
        val jsExecutor = driver as JavascriptExecutor
        val jsScript = "return arguments[0].querySelector('.modal-anchor')"
        return jsExecutor.executeScript(jsScript, startButton.shadowRoot) as WebElement
    }

    private fun <T> performAction(frameSync: Boolean = true, action: () -> T): T {
        if (!frameSync) driver.executeScript("flutter:setFrameSync", false, SET_FRAME_SYNC_MAX_WAIT_MILLIS)
        return try {
            action()
        } finally {
            if (!frameSync) driver.executeScript("flutter:setFrameSync", true, SET_FRAME_SYNC_MAX_WAIT_MILLIS)
        }
    }

    fun tapCoordinates(x: Int, y: Int) {
        Thread.sleep(PAGE_LOAD_TIMEOUT)
        try {
            // Create a PointerInput instance for touch gestures
            val finger = PointerInput(PointerInput.Kind.TOUCH, "finger")

            // Define the action sequence for a tap
            val tap = org.openqa.selenium.interactions.Sequence(finger, 0)
            tap.addAction(finger.createPointerMove(Duration.ZERO, PointerInput.Origin.viewport(), x, y))
            tap.addAction(finger.createPointerDown(PointerInput.MouseButton.LEFT.asArg()))
            tap.addAction(finger.createPointerUp(PointerInput.MouseButton.LEFT.asArg()))

            // Perform the action sequence
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

    fun enableNetworkConnection() {
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

    fun getTextFromElementContainingText(partialText: String): String? = withNativeContext {
        val element = try {
            findElementByPartialText(partialText)
        } catch (e: Exception) {
            println("Failed to get element text: ${e.message}")
            null
        }

        if (element == null) {
            throw NoSuchElementException("No element found containing: $partialText")
        }

        when (val platform = platformName()) {
            "ANDROID" -> element.getAttribute("contentDescription")
            "IOS" -> element.getAttribute("name")
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
    }

    fun clickElementContainingText(partialText: String) = withNativeContext {
        try {
            findElementByPartialText(partialText).click()
        } catch (e: Exception) {
            println("Failed to get element: ${e.message}")
        }
    }

    fun elementContainingTextVisible(partialText: String): Boolean = withNativeContext {
        try {
            findElementByPartialText(partialText).isDisplayed
        } catch (e: Exception) {
            println("Element not found or error occurred: ${e.message}")
            false
        }
    }

    private fun findElementByPartialText(partialText: String): WebElement {
        return when (val platform = platformName()) {
            "ANDROID" -> driver.findElement(
                AppiumBy.androidUIAutomator("new UiSelector().descriptionContains(\"$partialText\")")
            )
            "IOS" -> driver.findElement(
                By.xpath("//*[contains(@name, '$partialText')]")
            )
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
    }

    private fun <T> withNativeContext(block: () -> T): T {
        switchToNativeContext()
        return try {
            block()
        } finally {
            switchToAppContext()
        }
    }

    fun putAppInBackground(seconds: Int) {
        val driver = when (val platform = platformName()) {
            "ANDROID" -> driver as AndroidDriver
            "IOS" -> driver as IOSDriver
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
        switchToNativeContext()
        driver.runAppInBackground(Duration.ofSeconds(seconds.toLong()))
        switchToAppContext()
    }

    fun openUniversalLink(expiredUniversalLinkFromCameraApp: String) {
        val driver = driver as AppiumDriver
        switchToNativeContext()
        driver.get(expiredUniversalLinkFromCameraApp)
        switchToAppContext()
        Thread.sleep(SET_FRAME_SYNC_MAX_WAIT_MILLIS)
    }

    companion object {
        private const val SET_FRAME_SYNC_MAX_WAIT_MILLIS = 2000L
        private const val WAIT_FOR_ELEMENT_MAX_WAIT_MILLIS = 8000L
        private const val WAIT_FOR_CONTEXT_MAX_WAIT_MILLIS = 4000L
        private const val BROWSER_STARTUP_TIMEOUT = 2000L
        const val PAGE_LOAD_TIMEOUT = 4000L
        private const val MODAL_ANCHOR_DISPLAY_TIMEOUT = 500L

        private const val FLUTTER_APP_CONTEXT = "FLUTTER"
        private const val WEB_VIEW_CONTEXT_PREFIX = "WEBVIEW_"
        private const val NATIVE_APP_CONTEXT = "NATIVE_APP"

        private val browserStackUserName = EnvironmentUtil.getVar("BROWSERSTACK_USER")
        private val browserStackAccessKey = EnvironmentUtil.getVar("BROWSERSTACK_KEY")
        private const val BROWSERSTACK_ENDPOINT = "https://api.browserstack.com/app-automate/sessions/"
    }
}
