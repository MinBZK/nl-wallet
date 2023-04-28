package helper

import config.TestDataConfig.Companion.browserstackAccessKey
import config.TestDataConfig.Companion.browserstackUserName
import config.TestDataConfig.Companion.testDataConfig
import io.restassured.RestAssured

object Browserstack {
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
}