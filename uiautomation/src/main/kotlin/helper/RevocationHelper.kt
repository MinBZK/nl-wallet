package helper

import io.restassured.RestAssured
import io.restassured.http.ContentType
import org.json.JSONArray

class RevocationHelper {

    private val pidIssuerBaseUrl: String = System.getenv("INTERNAL_PID_ISSUER_URL")
    private val issuanceServerBaseUrl: String = System.getenv("INTERNAL_ISSUANCE_SERVER_URL")

    fun revokeAllNonRevokedPids() {
        revokeAllNonRevoked(pidIssuerBaseUrl)
    }

    fun revokeAllNonRevokedEeaCards() {
        revokeAllNonRevoked(issuanceServerBaseUrl)
    }

    private fun revokeAllNonRevoked(baseUrl: String) {

        val response = RestAssured.given()
            .baseUri(baseUrl)
            .contentType(ContentType.JSON)
            .accept(ContentType.JSON)
            .`when`()
            .get("/batch/")
            .then()
            .statusCode(200)
            .extract()
            .response()

        val batches = JSONArray(response.asString())

        val nonRevokedBatchIds = mutableListOf<String>()
        for (i in 0 until batches.length()) {
            val batch = batches.getJSONObject(i)
            val batchId = batch.getString("batch_id")
            val isRevoked = batch.getBoolean("is_revoked")

            if (!isRevoked) {
                nonRevokedBatchIds.add(batchId)
            }
        }

        RestAssured.given()
            .baseUri(baseUrl)
            .contentType(ContentType.JSON)
            .accept(ContentType.JSON)
            .body(JSONArray(nonRevokedBatchIds).toString())
            .`when`()
            .post("/revoke/")
            .then()
            .statusCode(200)
    }
}
