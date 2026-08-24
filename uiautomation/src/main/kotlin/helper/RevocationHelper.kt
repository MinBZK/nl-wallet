package helper

import io.restassured.RestAssured
import io.restassured.http.ContentType
import io.restassured.specification.ProxySpecification
import io.restassured.specification.RequestSpecification
import org.json.JSONArray
import org.json.JSONObject
import util.EnvironmentUtil
import java.net.URI

class RevocationHelper {

    private val pidIssuerBaseUrl: String = EnvironmentUtil.getVar("INTERNAL_PID_ISSUER_URL")
    private val issuanceServerBaseUrl: String = EnvironmentUtil.getVar("INTERNAL_ISSUANCE_SERVER_URL")
    private val walletProviderBaseUrl: String = EnvironmentUtil.getVar("INTERNAL_WALLET_PROVIDER_URL")

    // The INTERNAL_* endpoints live on the cluster-internal network. On the mac mini
    // iOS runner (which is outside the cluster) they are only reachable through the forward proxy.
    // RestAssured/Apache HttpClient does NOT honour the https_proxy env variable automatically.
    private val proxy: ProxySpecification? = proxyFromEnv()
    private val noProxyHosts: List<String> = noProxyFromEnv()

    private fun request(baseUrl: String): RequestSpecification {
        val spec = RestAssured.given().baseUri(baseUrl)
        return if (proxy != null && !isProxyExempt(baseUrl)) spec.proxy(proxy) else spec
    }

    private fun proxyFromEnv(): ProxySpecification? {
        val raw = EnvironmentUtil.getVar("https_proxy")
        if (raw.isBlank()) return null

        val uri = URI(raw)
        return ProxySpecification.host(uri.host).withPort(uri.port).withScheme(uri.scheme)
    }

    private fun noProxyFromEnv(): List<String> =
        EnvironmentUtil.getVar("no_proxy")
            .split(",")
            .map { it.trim().lowercase() }

    private fun isProxyExempt(baseUrl: String): Boolean {
        val host = (URI(baseUrl).host ?: return false).lowercase()
        return host in noProxyHosts
    }

    fun revokeAllNonRevokedPids() {
        revokeAllNonRevoked(pidIssuerBaseUrl)
    }

    fun revokeAllNonRevokedEeaCards() {
        revokeAllNonRevoked(issuanceServerBaseUrl)
    }

    private fun revokeAllNonRevoked(baseUrl: String) {

        val response = request(baseUrl)
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

        request(baseUrl)
            .contentType(ContentType.JSON)
            .accept(ContentType.JSON)
            .body(JSONArray(nonRevokedBatchIds).toString())
            .`when`()
            .post("revoke/")
            .then()
            .statusCode(200)
    }

    fun revokeAllActiveWallets() {
        val response = request(walletProviderBaseUrl)
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

        request(walletProviderBaseUrl)
            .contentType(ContentType.JSON)
            .accept(ContentType.JSON)
            .body(JSONArray(activeWalletIds).toString())
            .`when`()
            .post("/internal/revoke-wallets-by-id/")
            .then()
            .statusCode(200)
    }

    fun revokeWalletSolution() {
        request(walletProviderBaseUrl)
            .`when`()
            .post("/internal/revoke-solution/")
            .then()
            .statusCode(200)
    }

    fun restoreWalletSolution() {
        request(walletProviderBaseUrl)
            .`when`()
            .post("/internal/restore-solution/")
            .then()
            .statusCode(200)
    }

    fun revokeWalletByRecoveryCode(recoveryCode: String) {
        request(walletProviderBaseUrl)
            .contentType(ContentType.JSON)
            .accept(ContentType.JSON)
            .body(JSONObject.quote(recoveryCode))
            .`when`()
            .post("/internal/revoke-wallet-by-recovery-code/")
            .then()
            .statusCode(200)
    }

    fun deleteFromDenyList(recoveryCode: String) {
        request(walletProviderBaseUrl)
            .`when`()
            .delete("/internal/deny-list/$recoveryCode")
            .then()
            .statusCode(204)
    }
}
