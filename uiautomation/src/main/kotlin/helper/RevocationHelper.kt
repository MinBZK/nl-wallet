package helper

import io.restassured.RestAssured
import io.restassured.http.ContentType
import org.json.JSONArray
import org.json.JSONObject
import util.EnvironmentUtil

class RevocationHelper {

    private val pidIssuerBaseUrl: String = EnvironmentUtil.getVar("INTERNAL_PID_ISSUER_URL")
    private val issuanceServerBaseUrl: String = EnvironmentUtil.getVar("INTERNAL_ISSUANCE_SERVER_URL")
    private val walletProviderBaseUrl: String = EnvironmentUtil.getVar("INTERNAL_WALLET_PROVIDER_URL")

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
            .get("batch/")
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
            .post("revoke/")
            .then()
            .statusCode(200)
    }

    fun revokeAllActiveWallets() {
        val response = RestAssured.given()
            .baseUri(walletProviderBaseUrl)
            .accept(ContentType.JSON)
            .`when`()
            .get("/internal/wallet/")
            .then()
            .statusCode(200)
            .extract()
            .response()

        val wallets = JSONArray(response.asString())
        val activeWalletIds = wallets.mapNotNull { wallet ->
            if (wallet !is JSONObject?) return@mapNotNull null
            wallet.takeIf { it.getString("state") == "Active" }?.getString("wallet_id")
        }

        if (activeWalletIds.isEmpty()) return

        RestAssured.given()
            .baseUri(walletProviderBaseUrl)
            .contentType(ContentType.JSON)
            .accept(ContentType.JSON)
            .body(JSONArray(activeWalletIds).toString())
            .`when`()
            .post("/internal/revoke-wallets-by-id/")
            .then()
            .statusCode(200)
    }

    fun revokeWalletSolution() {
        RestAssured.given()
            .baseUri(walletProviderBaseUrl)
            .`when`()
            .post("/internal/revoke-solution/")
            .then()
            .statusCode(200)
    }

    fun restoreWalletSolution() {
        RestAssured.given()
            .baseUri(walletProviderBaseUrl)
            .`when`()
            .post("/internal/restore-solution/")
            .then()
            .statusCode(200)
    }

    fun revokeWalletByRecoveryCode(recoveryCode: String) {
        RestAssured.given()
            .baseUri(walletProviderBaseUrl)
            .contentType(ContentType.JSON)
            .accept(ContentType.JSON)
            .body(JSONObject.quote(recoveryCode))
            .`when`()
            .post("/internal/revoke-wallet-by-recovery-code/")
            .then()
            .statusCode(200)
    }

    fun deleteFromDenyList(recoveryCode: String) {
        RestAssured.given()
            .baseUri(walletProviderBaseUrl)
            .`when`()
            .delete("/internal/deny-list/$recoveryCode")
            .then()
            .statusCode(204)
    }
}
