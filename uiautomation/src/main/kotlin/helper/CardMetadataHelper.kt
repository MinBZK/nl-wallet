package helper

import helper.FileUtils.getProjectFile
import org.json.JSONObject
import util.TestInfoHandler.Companion.language
import util.TestInfoHandler.Companion.locale
import java.io.File

class CardMetadataHelper {

    private val pidTAS: JSONObject by lazy {
        val jsonContent = File(PID_CARD_METADATA_FILE_PATH).readText(Charsets.UTF_8)
        JSONObject(jsonContent)
    }

    private val addressTAS: JSONObject by lazy {
        val jsonContent = File(ADDRESS_CARD_METADATA_FILE_PATH).readText(Charsets.UTF_8)
        JSONObject(jsonContent)
    }

    fun getPidVCT(): String {
        return pidTAS.optString("vct", null)
    }

    fun getAddressACT(): String {
        return addressTAS.optString("vct", null)
    }

    fun getPidDisplayName(): String {
        val displayArray = pidTAS.optJSONArray("display") ?: throw Exception("Missing 'display' array in PID tas")
        for (i in 0 until displayArray.length()) {
            val display = displayArray.getJSONObject(i)
            if (display.getString("lang") == "$language-$locale") {
                return display.optString("name", null)
            }
        }
        throw Exception("cannot find displayName")
    }

    fun getAddressDisplayName(): String {
        val displayArray = addressTAS.optJSONArray("display") ?: throw Exception("Missing 'display' array in address tas")
        for (i in 0 until displayArray.length()) {
            val display = displayArray.getJSONObject(i)
            if (display.getString("lang") == "$language-$locale") {
                return display.optString("name", null)
            }
        }
        throw Exception("cannot find address displayName")
    }

    fun getPidClaimLabel(pathValue: String): String {
        val claims = pidTAS.getJSONArray("claims")
        for (i in 0 until claims.length()) {
            val claim = claims.getJSONObject(i)
            val pathArray = claim.getJSONArray("path")

            if (pathArray.length() == 1 && pathArray.getString(0) == pathValue) {
                val displayArray = claim.getJSONArray("display")
                for (j in 0 until displayArray.length()) {
                    val display = displayArray.getJSONObject(j)
                    if (display.getString("lang") == "$language-$locale") {
                        return display.getString("label")
                    }
                }
            }
        }
        throw Exception("cannot find claim label")
    }

    fun getAddressClaimLabel(pathValue: String): String {
        val claims = addressTAS.getJSONArray("claims")
        for (i in 0 until claims.length()) {
            val claim = claims.getJSONObject(i)
            val pathArray = claim.getJSONArray("path")

            if (pathArray.length() == 1 && pathArray.getString(0) == pathValue) {
                val displayArray = claim.getJSONArray("display")
                for (j in 0 until displayArray.length()) {
                    val display = displayArray.getJSONObject(j)
                    if (display.getString("lang") == "$language-$locale") {
                        return display.getString("label")
                    }
                }
            }
        }
        throw Exception("cannot find address claim label")
    }

    companion object {
        val PID_CARD_METADATA_FILE_PATH = getProjectFile("scripts/devenv/eudi:pid:nl:1.json")
        val ADDRESS_CARD_METADATA_FILE_PATH = getProjectFile("scripts/devenv/eudi:pid-address:nl:1.json")
    }
}

