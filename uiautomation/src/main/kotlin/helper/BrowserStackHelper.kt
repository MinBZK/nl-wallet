package helper

import io.restassured.RestAssured
import io.restassured.http.ContentType

object BrowserStackHelper {

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
