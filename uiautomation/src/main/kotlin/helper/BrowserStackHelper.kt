package helper

import com.codeborne.selenide.Selenide
import com.codeborne.selenide.WebDriverRunner.getWebDriver
import data.TestConfigRepository.Companion.testConfig
import io.restassured.RestAssured
import io.restassured.http.ContentType
import org.openqa.selenium.JavascriptExecutor
import org.openqa.selenium.remote.RemoteWebDriver
import util.EnvironmentUtil
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter

object BrowserStackHelper {
    val buildName = generateBuildName()

    fun getAppUrl(endpoint: String, userName: String, accessKey: String, customId: String): String {
        return RestAssured
            .given()
            .auth().basic(userName, accessKey)
            .`when`()
            .get(endpoint + customId)
            .then()
            .statusCode(200)
            .extract()
            .path("[0].app_url")
    }

    fun markTest(status: String) {
        if (!testConfig.remote) return

        val jse: JavascriptExecutor = getWebDriver() as RemoteWebDriver
        jse.executeScript("browserstack_executor: {\"action\": \"setSessionStatus\", \"arguments\": {\"status\": \"$status\"}}")
        Selenide.closeWebDriver()
    }

    private fun generateBuildName(): String {
        val currentDateTime = LocalDateTime.now()
        val formatter = EnvironmentUtil.getVar("BUILD_NAME_DATE_FORMAT_OVERRIDE").takeIf { it.isNotEmpty() }
            ?.let { DateTimeFormatter.ofPattern(it) }
            ?: DateTimeFormatter.ofPattern("dd/MM-HH:mm:ss")
        val formattedDateTime = currentDateTime.format(formatter)
        return "build-$formattedDateTime"
    }

    fun setNetwork(endpoint: String, userName: String, accessKey: String, sessionId: String, networkProfile: String) {
        RestAssured
            .given()
            .contentType(ContentType.JSON)
            .auth().basic(userName, accessKey)
            .body("{\"networkProfile\":\"$networkProfile\"}")
            .put("$endpoint$sessionId/update_network.json")
            .then()
            .statusCode(200)
    }
}
