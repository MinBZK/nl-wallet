package helper

import com.codeborne.selenide.Selenide
import com.codeborne.selenide.WebDriverRunner.getWebDriver
import config.TestDataConfig
import config.TestDataConfig.Companion.browserstackAccessKey
import config.TestDataConfig.Companion.browserstackUserName
import config.TestDataConfig.Companion.testDataConfig
import io.restassured.RestAssured
import org.openqa.selenium.JavascriptExecutor
import org.openqa.selenium.remote.RemoteWebDriver
import util.EnvUtilities.getEnvVar
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter

object Browserstack {
    val buildName = generateBuildName()
    fun videoUrl(sessionId: String?): String {

        val url = testDataConfig.let { String.format(it.sessionUrl, sessionId) }
        return RestAssured.given()
            .auth().basic(browserstackUserName, browserstackAccessKey)
            .log().all()
            .`when`()[url]
            .then()
            .log().all()
            .statusCode(200)
            .extract()
            .path("automation_session.video_url")
    }

    fun getAppUrl(platform: String?): String {
        val appUrl = testDataConfig.uploadedApp + platform
        return RestAssured.given()
            .auth().basic(browserstackUserName, browserstackAccessKey)
            .log().all()
            .`when`()[appUrl]
            .then()
            .log().all()
            .statusCode(200)
            .extract()
            .path("[0].app_url")
    }

    private fun generateBuildName(): String {
        val currentDateTime = LocalDateTime.now()
        val formatter = getEnvVar("BUILD_NAME_DATE_FORMAT_OVERRIDE").takeIf { it.isNotEmpty() }
            ?.let { DateTimeFormatter.ofPattern(it) }
            ?: DateTimeFormatter.ofPattern("dd/MM-HH:mmss")
        val formattedDateTime = currentDateTime.format(formatter)
        return "build-$formattedDateTime"
    }

    fun markTest(status: String) {
        if (testDataConfig.remoteOrLocal != TestDataConfig.RemoteOrLocal.remote) return

        val jse: JavascriptExecutor = getWebDriver() as RemoteWebDriver
        jse.executeScript("browserstack_executor: {\"action\": \"setSessionStatus\", \"arguments\": {\"status\": \"$status\"}}")
        Selenide.closeWebDriver()
    }
}
