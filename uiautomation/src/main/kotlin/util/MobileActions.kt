package util

import com.codeborne.selenide.WebDriverRunner.getWebDriver
import com.google.zxing.BinaryBitmap
import com.google.zxing.MultiFormatReader
import com.google.zxing.client.j2se.BufferedImageLuminanceSource
import com.google.zxing.common.HybridBinarizer
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
import org.openqa.selenium.OutputType
import org.openqa.selenium.TakesScreenshot
import org.openqa.selenium.WebElement
import org.openqa.selenium.interactions.PointerInput
import org.openqa.selenium.interactions.PointerInput.Origin
import org.openqa.selenium.remote.RemoteWebDriver
import org.openqa.selenium.remote.RemoteWebElement
import org.openqa.selenium.support.ui.ExpectedConditions
import org.openqa.selenium.support.ui.WebDriverWait
import java.io.File
import java.io.IOException
import java.time.Duration
import javax.imageio.ImageIO

open class MobileActions {

    val driver = getWebDriver() as RemoteWebDriver

    protected val l10n = LocalizationHelper()
    protected val cardMetadata = TasDataHelper()
    protected val organizationAuthMetadata = OrganizationAuthMetadataHelper()

    private fun quoteForAndroid(s: String): String =
        "\"" + s.replace("\\", "\\\\").replace("\"", "\\\"") + "\""

    private fun quoteForIos(s: String): String =
        "'" + s.replace("\\", "\\\\").replace("'", "\\'") + "'"

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

    protected fun findWebElement(locator: By): WebElement {
        val wait = WebDriverWait(driver, Duration.ofMillis(WAIT_FOR_ELEMENT_MAX_WAIT_MILLIS))
        wait.until(ExpectedConditions.visibilityOfElementLocated(locator))
        return driver.findElement(locator)
    }

    protected fun scrollToWebElement(locator: By): WebElement {
        val wait = WebDriverWait(driver, Duration.ofMillis(WAIT_FOR_ELEMENT_MAX_WAIT_MILLIS))
        val element = wait.until(ExpectedConditions.presenceOfElementLocated(locator))
        (driver as JavascriptExecutor).executeScript(
            "arguments[0].scrollIntoView({behavior: 'smooth', block: 'center'});",
            element
        )
        wait.until(ExpectedConditions.visibilityOf(element))
        return element
    }

    protected fun getTopLeftOfElementContainingText(text: String): Pair<Double, Double>? =
        try {
            val element = findElementByPartialText(text)
            val r = element.rect
            Pair(r.x.toDouble(), r.y.toDouble())
        } catch (_: Exception) {
            null
        }

    private fun findIosScrollView(): RemoteWebElement? =
        driver.findElements(AppiumBy.iOSClassChain("**/XCUIElementTypeScrollView[1]"))
            .firstOrNull() as? RemoteWebElement

    private fun iosSwipeArgs(direction: String): Map<String, Any> {
        val args = mutableMapOf<String, Any>("direction" to direction)
        findIosScrollView()?.let { args["element"] = it.id }
        return args
    }

    fun scrollToElementWithText(text: String): WebElement {
        return when (val platform = platformName()) {
            "ANDROID" -> {
                val quotedText = quoteForAndroid(text)
                driver.findElement(
                    AppiumBy.androidUIAutomator(
                        "new UiScrollable(new UiSelector().scrollable(true))" +
                            ".scrollIntoView(new UiSelector().description($quotedText))"
                    )
                )
            }
            "IOS" -> {
                val quotedText = quoteForIos(text)
                val predicate = "name == $quotedText"

                repeat(8) { // cap attempts to avoid infinite loops
                    val matches = driver.findElements(AppiumBy.iOSNsPredicateString(predicate))
                    matches.firstOrNull { it.isDisplayed }?.let { return it }
                    (driver as JavascriptExecutor).executeScript(
                        "mobile: swipe",
                        iosSwipeArgs("up")
                    )
                }
                throw NoSuchElementException("Couldn't bring '$text' into view")
            }
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
    }

    fun scrollToElementContainingText(text: String): WebElement {
        return when (val platform = platformName()) {
            "ANDROID" -> {
                val quotedText = quoteForAndroid(text)
                driver.findElement(
                    AppiumBy.androidUIAutomator(
                        "new UiScrollable(new UiSelector().scrollable(true))" +
                            ".scrollIntoView(new UiSelector().descriptionContains($quotedText))"
                    )
                )
            }
            "IOS" -> {
                val quotedText = quoteForIos(text)
                val predicate = "name CONTAINS $quotedText"

                repeat(8) {
                    val matches = driver.findElements(AppiumBy.iOSNsPredicateString(predicate))
                    matches.firstOrNull { it.isDisplayed }?.let { return it }
                    (driver as JavascriptExecutor).executeScript(
                        "mobile: swipe",
                        iosSwipeArgs("up")
                    )
                }
                throw NoSuchElementException("Couldn't bring element containing '$text' into view")
            }
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
    }

    fun scrollToElementContainingTexts(partialTexts: List<String>) {
        when (val platform = platformName()) {
            "ANDROID" -> {
                val regexPattern = ".*" + partialTexts.joinToString(".*") { Regex.escape(it) } + ".*"
                val quotedPattern = "\"${regexPattern.replace("\"", "\\\"")}\""
                driver.findElement(
                    AppiumBy.androidUIAutomator(
                        "new UiScrollable(new UiSelector().scrollable(true))" +
                            ".scrollIntoView(new UiSelector().descriptionMatches($quotedPattern))"
                    )
                ) ?: throw NoSuchElementException("Element containing texts $partialTexts not found")
            }
            "IOS" -> {
                val predicate = partialTexts.joinToString(" AND ") { partialText ->
                    val quotedText = quoteForIos(partialText)
                    "name CONTAINS $quotedText"
                }
                val scrollArgs = mutableMapOf<String, Any>("predicateString" to predicate, "toVisible" to true)
                findIosScrollView()?.let { scrollArgs["element"] = it.id }
                (driver as JavascriptExecutor).executeScript(
                    "mobile: scroll",
                    scrollArgs
                ) ?: throw NoSuchElementException("Element containing texts $partialTexts not found")
            }
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
    }

    fun scrollDown(pixels: Int, durationMs: Int = 300) {
        val driver = when (val platform = platformName()) {
            "ANDROID" -> driver as AndroidDriver
            "IOS" -> driver as IOSDriver
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }

        val size = driver.manage().window().size
        val width = size.width
        val height = size.height

        val startX = width / 2
        val startY = (height * 0.60).toInt()
        val endX   = startX
        val endY   = startY - pixels  // Scroll down by moving finger up

        val finger = PointerInput(PointerInput.Kind.TOUCH, "finger")
        val swipe = org.openqa.selenium.interactions.Sequence(finger, 1)
            .addAction(finger.createPointerMove(Duration.ZERO, Origin.viewport(), startX, startY))
            .addAction(finger.createPointerDown(PointerInput.MouseButton.LEFT.asArg()))
            .addAction(finger.createPointerMove(Duration.ofMillis(durationMs.toLong()), Origin.viewport(), endX, endY))
            .addAction(finger.createPointerUp(PointerInput.MouseButton.LEFT.asArg()))

        driver.perform(listOf(swipe))
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
        val startY = (height * 0.60).toInt()   // near bottom
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
                val handles = driver.contextHandles
                handles.firstOrNull { it.startsWith(WEB_VIEW_CONTEXT_PREFIX) }
            }

            // Switch to the web view context
            driver.context(contextHandle)

            // Explicit timeout; waiting for the browser to be fully started and the viewport stabilized.
            // This fixes the issue where the (Chrome) browser viewport flickers back and forth between
            // the loaded web page and the browser startup screen shortly after browser startup.
            Thread.sleep(1000)

            // Switch to the last window handle (a.k.a. tab)
            val windowHandles = (driver as AppiumDriver).windowHandles
            driver.switchTo().window(windowHandles.last())
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
        Thread.sleep(1000)
    }

    protected fun getWebModalAnchor(): WebElement {
        Thread.sleep(BROWSER_STARTUP_TIMEOUT)
        when (val platform = platformName()) {
            "ANDROID" -> {
                val startButton = driver.findElement(By.tagName("nl-wallet-button"))
                val jsExecutor = driver as JavascriptExecutor
                val jsScript = "return arguments[0].querySelector('.modal-anchor')"
                return jsExecutor.executeScript(jsScript, startButton.shadowRoot) as WebElement
            }
            "IOS" -> {
                val wait = WebDriverWait(driver, Duration.ofSeconds(10))
                val startButton = wait.until(ExpectedConditions.presenceOfElementLocated(By.tagName("nl-wallet-button")))

                val js = driver as JavascriptExecutor
                val modalAnchor = js.executeScript(
                    """
                    const host = arguments[0];
                    if (!host?.shadowRoot) return null;
                    return host.shadowRoot.querySelector('.modal-anchor');
                    """.trimIndent(),
                    startButton
                ) as WebElement
                return modalAnchor
            }
            else -> {
                throw IllegalArgumentException("Unsupported platform: $platform")
            }
        }
    }

    fun platformName() = driver.capabilities.platformName?.name ?: throw IllegalStateException("No platform name")

    fun getElementText(element: WebElement): String {
        return when (platformName()) {
            "ANDROID" -> element.getAttribute("contentDescription")
            "IOS" -> element.getAttribute("name")
            else -> element.text
        } ?: element.text
    }

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
        findElementByPartialText(partialText).click()
    }

    fun clickElementWithText(text: String, timeoutInSeconds: Long = 5) {
        findElementByText(text, timeoutInSeconds).click()
    }

    fun elementContainingTextVisible(partialText: String): Boolean {
        return try {
            findElementByPartialText(partialText).isDisplayed
        } catch (e: Exception) {
            println("Element not found or error occurred: ${e.message}")
            false
        }
    }

    fun elementContainingTextsVisible(partialTexts: List<String>): Boolean {
        return try {
            findElementByPartialTexts(partialTexts).isDisplayed
        } catch (e: Exception) {
            println("Element not found or error occurred: ${e.message}")
            false
        }
    }

    fun elementWithTextVisible(text: String, timeoutInSeconds: Long = 5): Boolean {
        return try {
            findElementByText(text, timeoutInSeconds).isDisplayed
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

    fun getTextFromAllChildElementsFromElementWithText(parentText: String): String {
        val parentElement = findElementByText(parentText)

        val childElements = parentElement.findElements(By.xpath(".//*"))

        return childElements.joinToString("") { element ->
            when (val platform = platformName()) {
                "ANDROID" -> element.getAttribute("contentDescription") ?: ""
                "IOS" -> element.getAttribute("name") ?: ""
                else -> ""
            }
        }
    }

    private fun findElementByPartialText(partialText: String, timeoutInSeconds: Long = 5): WebElement {
        val wait = WebDriverWait(driver, Duration.ofSeconds(timeoutInSeconds))
        return when (val platform = platformName()) {
            "ANDROID" -> {
                val quotedText = quoteForAndroid(partialText)
                wait.until(
                    ExpectedConditions.presenceOfElementLocated(
                        AppiumBy.androidUIAutomator("new UiSelector().descriptionContains($quotedText)")
                    )
                )
            }
            "IOS" -> {
                val quotedText = quoteForIos(partialText)
                wait.until(
                    ExpectedConditions.presenceOfElementLocated(
                        By.xpath("//*[contains(@name, $quotedText)]")
                    )
                )
            }
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
    }

    private fun findElementByPartialTexts(
        partialTexts: List<String>,
        timeoutInSeconds: Long = 5
    ): WebElement {
        val wait = WebDriverWait(driver, Duration.ofSeconds(timeoutInSeconds))
        return when (val platform = platformName()) {
            "ANDROID" -> {
                val regexPattern = ".*" + partialTexts.joinToString(".*") { Regex.escape(it) } + ".*"
                val quotedPattern = "\"${regexPattern.replace("\"", "\\\"")}\""
                wait.until(
                    ExpectedConditions.presenceOfElementLocated(
                        AppiumBy.androidUIAutomator("new UiSelector().descriptionMatches($quotedPattern)")
                    )
                )
            }
            "IOS" -> {
                val xpathConditions = partialTexts.joinToString(" and ") { partialText ->
                    val quotedText = quoteForIos(partialText)
                    "contains(@name, $quotedText)"
                }
                wait.until(
                    ExpectedConditions.presenceOfElementLocated(
                        By.xpath("//*[$xpathConditions]")
                    )
                )
            }
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
    }

    fun findElementByPartialTextExcludingText(
        includeText: String,
        excludeText: String,
        timeoutInSeconds: Long = 5
    ): WebElement {
        val wait = WebDriverWait(driver, Duration.ofSeconds(timeoutInSeconds))
        return when (val platform = platformName()) {
            "ANDROID" -> {
                val escapedInclude = Regex.escape(includeText)
                val escapedExclude = Regex.escape(excludeText)
                val regexPattern = "(?s)^(?!.*$escapedExclude).*$escapedInclude.*$"
                val quotedPattern = "\"${regexPattern.replace("\"", "\\\"")}\""
                wait.until(
                    ExpectedConditions.presenceOfElementLocated(
                        AppiumBy.androidUIAutomator("new UiSelector().descriptionMatches($quotedPattern)")
                    )
                )
            }
            "IOS" -> {
                val quotedInclude = quoteForIos(includeText)
                val quotedExclude = quoteForIos(excludeText)
                wait.until(
                    ExpectedConditions.presenceOfElementLocated(
                        By.xpath("//*[contains(@name, $quotedInclude) and not(contains(@name, $quotedExclude))]")
                    )
                )
            }
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
    }

    fun elementContainingTextExcludingTextVisible(includeText: String, excludeText: String): Boolean {
        return try {
            findElementByPartialTextExcludingText(includeText, excludeText).isDisplayed
        } catch (e: Exception) {
            println("Element not found or error occurred: ${e.message}")
            false
        }
    }

    private fun findElementByText(text: String, timeoutInSeconds: Long = 5): WebElement {
        val wait = WebDriverWait(driver, Duration.ofSeconds(timeoutInSeconds))
        return when (val platform = platformName()) {
            "ANDROID" -> {
                val quotedText = quoteForAndroid(text)
                wait.until(
                    ExpectedConditions.presenceOfElementLocated(
                        AppiumBy.androidUIAutomator("new UiSelector().description($quotedText)")
                    )
                )
            }
            "IOS" -> {
                val quotedText = quoteForIos(text)
                wait.until(
                    ExpectedConditions.presenceOfElementLocated(
                        AppiumBy.iOSNsPredicateString("name == $quotedText")
                    )
                )
            }
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
    }

    private fun findElementByDescendantElementText(
        descendantElementText: String,
        elementText: String
    ): WebElement {
        return when (val platform = platformName()) {
            "ANDROID" -> {
                val quotedElementText = quoteForAndroid(elementText)
                val quotedDescendantElementText = quoteForAndroid(descendantElementText)
                driver.findElement(By.xpath("//*[@content-desc=$quotedElementText and .//*[@content-desc=$quotedDescendantElementText]]"))
            }
            "IOS" -> {
                val quotedElementText = quoteForIos(elementText)
                val quotedDescendantElementText = quoteForIos(descendantElementText)
                driver.findElement(By.xpath("//*[@name=$quotedElementText and .//*[@name=$quotedDescendantElementText]]"))
            }
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
    }

    fun findElementByPartialTextAndPartialSiblingText(
        text: String,
        siblingText: String,
        timeoutInSeconds: Long = 5
    ): WebElement {
        val wait = WebDriverWait(driver, Duration.ofSeconds(timeoutInSeconds))
        return when (val platform = platformName()) {
            "ANDROID" -> {
                val quotedText = quoteForAndroid(text)
                val quotedSibling = quoteForAndroid(siblingText)

                val xpath = "//*[contains(@content-desc, $quotedText) and ../*[contains(@content-desc, $quotedSibling)]]"
                wait.until(ExpectedConditions.presenceOfElementLocated(AppiumBy.xpath(xpath)))
            }

            "IOS" -> {
                val quotedText = quoteForIos(text)
                val quotedSibling = quoteForIos(siblingText)

                val xpath = "//*[contains(@name, $quotedText) and ../*[contains(@name, $quotedSibling)]]"
                wait.until(ExpectedConditions.presenceOfElementLocated(By.xpath(xpath)))
            }

            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
    }

    fun openApp() {
        val driver = when (val platform = platformName()) {
            "ANDROID" -> driver as AndroidDriver
            "IOS" -> driver as IOSDriver
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
        driver.activateApp(testConfig.appIdentifier)
        Thread.sleep(1000)
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

    fun decodeQrFromBytes(bytes: ByteArray): String {
        val image = ImageIO.read(bytes.inputStream())
        val binaryBitmap = BinaryBitmap(HybridBinarizer(BufferedImageLuminanceSource(image)))
        return MultiFormatReader().decode(binaryBitmap).text
    }

    fun takeScreenshotOfElement(text: String): ByteArray {
        val element = when (val platform = platformName()) {
            "ANDROID" -> findWebElement(By.xpath("//*[@content-desc=${quoteForAndroid(text)}]"))
            "IOS" -> findWebElement(AppiumBy.iOSNsPredicateString("name == ${quoteForIos(text)}"))
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
        return (element as TakesScreenshot).getScreenshotAs(OutputType.BYTES)
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

    fun openLink(url: String) {
        val driver = driver as AppiumDriver
        driver.get(url)
        Thread.sleep(SET_FRAME_SYNC_MAX_WAIT_MILLIS)
    }

    fun startMockBleReaderApp(
        mdocQrString: String,
        timeoutSeconds: Int = 60,
        readerCaCrtFile: String? = null,
        readerCaKeyFile: String? = null,
        readerAuthFile: String? = null,
        waitForDeviceResponse: Boolean = false,
    ): Process {
        val qrPayload = mdocQrString.removePrefix("mdoc:")
        val scriptPath = File("../scripts/close_proximity/disclosure_mac_reader.swift").canonicalPath
        val cmd = mutableListOf("swift", scriptPath, "--qr-code", qrPayload, "--timeout", timeoutSeconds.toString())
        if (readerCaCrtFile != null && readerCaKeyFile != null && readerAuthFile != null) {
            cmd += listOf("--reader-ca-crt-file", readerCaCrtFile, "--reader-ca-key-file", readerCaKeyFile, "--reader-auth-file", readerAuthFile)
        }
        if (waitForDeviceResponse) {
            cmd += "--print-device-response-hex"
        }
        return ProcessBuilder(cmd)
            .directory(File("..").canonicalFile)
            .redirectErrorStream(true)
            .start()
    }

    fun openUrlInBrowser(url: String) {
        when (val platform = platformName()) {
            "ANDROID" -> (driver as JavascriptExecutor).executeScript(
                "mobile: deepLink",
                mapOf("url" to url, "package" to "com.android.chrome"),
            )
            "IOS" -> (driver as JavascriptExecutor).executeScript(
                "mobile: safari launch",
                mapOf("url" to url),
            )
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
        Thread.sleep(SET_FRAME_SYNC_MAX_WAIT_MILLIS)
    }

    fun closeApp() {
        val driver = when (val platform = platformName()) {
            "ANDROID" -> driver as AndroidDriver
            "IOS" -> driver as IOSDriver
            else -> throw IllegalArgumentException("Unsupported platform: $platform")
        }
        driver.terminateApp(testConfig.appIdentifier)
    }

    companion object {
        const val SET_FRAME_SYNC_MAX_WAIT_MILLIS = 2000L
        private const val WAIT_FOR_ELEMENT_MAX_WAIT_MILLIS = 8000L
        private const val WAIT_FOR_CONTEXT_MAX_WAIT_MILLIS = 4000L
        private const val BROWSER_STARTUP_TIMEOUT = 2000L
        const val DEFAULT_RESET_SLEEP = 10_000L

        private const val WEB_VIEW_CONTEXT_PREFIX = "WEBVIEW_"
        private const val NATIVE_APP_CONTEXT = "NATIVE_APP"

        private val browserStackUserName = EnvironmentUtil.getVar("BROWSERSTACK_USER")
        private val browserStackAccessKey = EnvironmentUtil.getVar("BROWSERSTACK_KEY")
        private const val BROWSERSTACK_ENDPOINT = "https://api.browserstack.com/app-automate/sessions/"
    }
}

fun Process.captureOutput(): StringBuffer {
    val buffer = StringBuffer()
    Thread {
        inputStream.bufferedReader().forEachLine { line ->
            println(line)
            buffer.appendLine(line)
        }
    }.also { it.isDaemon = true; it.start() }
    return buffer
}
