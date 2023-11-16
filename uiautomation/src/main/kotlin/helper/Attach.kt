package helper

import com.codeborne.selenide.WebDriverRunner
import io.qameta.allure.Attachment
import org.openqa.selenium.OutputType
import org.openqa.selenium.TakesScreenshot
import org.openqa.selenium.remote.RemoteWebDriver
import java.time.ZoneId
import java.time.ZonedDateTime
import java.time.format.DateTimeFormatter

object Attach {
    private val driver = WebDriverRunner.getWebDriver() as RemoteWebDriver

    // Attaches the given message to the test report
    @Attachment(value = "{attachName}", type = "text/plain")
    fun attachAsText(attachName: String?, message: String): String {
        return message
    }

    // Returns the page source of the current WebDriver session
    @Attachment(value = "Page source", type = "text/plain")
    fun pageSource(): Any? {
        return driver.executeScript("flutter:getRenderTree")
    }

    // Takes a screenshot and attaches it to the test report as a PNG image
    @Attachment(value = "{attachName}", type = "image/png")
    fun screenshotAs(attachName: String?): ByteArray {
        return (WebDriverRunner.getWebDriver() as TakesScreenshot).getScreenshotAs(OutputType.BYTES)
    }

    // Add timestamp to the taken screenshot.
    fun screenshotWithTimeStamp() {
        val screenshotAlias = generateScreenshotAlias()
        screenshotAs(screenshotAlias)
    }

    // Returns a video that can be used to play a video recording of a Browserstack session with the given session ID.
    @Attachment(value = "Video", type = "text/html", fileExtension = ".html")
    fun video(sessionId: String?): String {
        return ("<html><body><video width='100%' height='100%' controls autoplay><source src='"
            + BrowserStackHelper.getVideoUrl(sessionId)
            ) + "' type='video/mp4'></video></body></html>"
    }

    // Get the current sessionId
    fun sessionId(): String {
        return (WebDriverRunner.getWebDriver() as RemoteWebDriver).sessionId.toString()
    }

    // This function generates a screenshot containing the current date and time in the format
    private fun generateScreenshotAlias(): String {
        return String.format(
            "Screenshot %s",
            ZonedDateTime
                .now(
                    ZoneId.of("Europe/Amsterdam")
                )
                .format(
                    DateTimeFormatter.ofPattern("uuuu.MM.dd.HH.mm.ss")
                )
        )
    }
}
