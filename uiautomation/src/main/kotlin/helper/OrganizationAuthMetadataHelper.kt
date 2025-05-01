package helper

import helper.FileUtils.getProjectFile
import org.json.JSONObject
import util.TestInfoHandler.Companion.language
import java.io.File

class OrganizationAuthMetadataHelper {

    private val amsterdamReaderAuthJSON: JSONObject by lazy {
        val jsonContent = File(AMSTERDAM_READER_AUTH_JSON_FILE_PATH).readText(Charsets.UTF_8)
        JSONObject(jsonContent)
    }

    private val xyzReaderAuthJSON: JSONObject by lazy {
        val jsonContent = File(XYZ_READER_AUTH_JSON_FILE_PATH).readText(Charsets.UTF_8)
        JSONObject(jsonContent)
    }

    private val mokeybikeReaderAuthJSON: JSONObject by lazy {
        val jsonContent = File(MONKEYBIKE_READER_AUTH_JSON_FILE_PATH).readText(Charsets.UTF_8)
        JSONObject(jsonContent)
    }

    private val marketplaceReaderAuthJSON: JSONObject by lazy {
        val jsonContent = File(MARKETPLACE_READER_AUTH_JSON_FILE_PATH).readText(Charsets.UTF_8)
        JSONObject(jsonContent)
    }

    private val rvigIssuerAuthJSON: JSONObject by lazy {
        val jsonContent = File(RVIG_ISSUER_AUTH_JSON_FILE_PATH).readText(Charsets.UTF_8)
        JSONObject(jsonContent)
    }

    fun getAttributeValueForOrganization(attributePath: String, rp: Organization): String {
        val json = when (rp) {
            Organization.AMSTERDAM -> amsterdamReaderAuthJSON
            Organization.XYZ -> xyzReaderAuthJSON
            Organization.MONKEYBIKE -> mokeybikeReaderAuthJSON
            Organization.MARKETPLACE -> marketplaceReaderAuthJSON
            Organization.RVIG -> rvigIssuerAuthJSON
        }

        val pathParts = attributePath.split(".")
        var current: Any = json

        for (part in pathParts) {
            if (current is JSONObject && current.has(part)) {
                current = current.get(part)
            } else {
                throw IllegalArgumentException("Invalid attribute path: '$attributePath'. Missing part: '$part'")
            }
        }

        if (current is JSONObject && current.has(language)) {
            return current.getString(language)
        }
        return current.toString()
    }

    enum class Organization {
        AMSTERDAM,
        XYZ,
        MONKEYBIKE,
        MARKETPLACE,
        RVIG
    }

    companion object {
        val AMSTERDAM_READER_AUTH_JSON_FILE_PATH = getProjectFile("scripts/devenv/mijn_amsterdam_reader_auth.json")
        val XYZ_READER_AUTH_JSON_FILE_PATH = getProjectFile("scripts/devenv/xyz_bank_reader_auth.json")
        val MONKEYBIKE_READER_AUTH_JSON_FILE_PATH = getProjectFile("scripts/devenv/monkey_bike_reader_auth.json")
        val MARKETPLACE_READER_AUTH_JSON_FILE_PATH = getProjectFile("scripts/devenv/online_marketplace_reader_auth.json")
        val RVIG_ISSUER_AUTH_JSON_FILE_PATH = getProjectFile("scripts/devenv/rvig_issuer_auth.json")
    }
}
