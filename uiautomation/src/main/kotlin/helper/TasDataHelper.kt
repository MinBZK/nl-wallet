package helper

import helper.FileUtils.getProjectFile
import org.json.JSONArray
import org.json.JSONObject
import util.TestInfoHandler.Companion.language
import util.TestInfoHandler.Companion.locale
import java.io.File

class TasDataHelper {

    private val extendedPidTAS: JSONObject by lazy {
        val jsonContent = File(getExtendedPidCardMetadataPath()).readText(Charsets.UTF_8)
        JSONObject(jsonContent)
    }

    private val basePidTAS: JSONObject by lazy {
        val jsonContent = File(getBasePidCardMetadataPath()).readText(Charsets.UTF_8)
        JSONObject(jsonContent)
    }

    private val extendedAddressTAS: JSONObject by lazy {
        val jsonContent = File(getExtendedAddressCardMetadataPath()).readText(Charsets.UTF_8)
        JSONObject(jsonContent)
    }

    private val baseAddressTAS: JSONObject by lazy {
        val jsonContent = File(getBaseAddressCardMetadataPath()).readText(Charsets.UTF_8)
        JSONObject(jsonContent)
    }

    private val diplomaTAS: JSONObject by lazy {
        val jsonContent = File(getDiplomaCardMetadataPath()).readText(Charsets.UTF_8)
        JSONObject(jsonContent)
    }

    private val insuranceTAS: JSONObject by lazy {
        val jsonContent = File(getInsuranceCardMetadataPath()).readText(Charsets.UTF_8)
        JSONObject(jsonContent)
    }

    private fun getExtendedPidCardMetadataPath() = getProjectFile("scripts/devenv/eudi:pid:nl:1.json")

    private fun getBasePidCardMetadataPath() = getProjectFile("scripts/devenv/eudi:pid:1.json")

    private fun getExtendedAddressCardMetadataPath() = getProjectFile("scripts/devenv/eudi:pid-address:nl:1.json")

    private fun getBaseAddressCardMetadataPath() = getProjectFile("scripts/devenv/eudi:pid-address:1.json")

    private fun getDiplomaCardMetadataPath() = getProjectFile("scripts/devenv/com.example.degree.json")

    private fun getInsuranceCardMetadataPath() = getProjectFile("scripts/devenv/com.example.insurance.json")

    fun getPidVCT(): String {
        val vct = extendedPidTAS.optString("vct")
        if (vct.isNullOrEmpty()) {
            throw Exception("Cannot find 'vct' field in extended PID TAS file")
        }
        return vct
    }

    fun getPidDisplayName() = findDisplayName(extendedPidTAS, basePidTAS)

    fun getPidClaimLabel(pathValue: String): String {
        return findClaimLabel(extendedPidTAS, basePidTAS, pathValue = pathValue)
    }

    fun getAddressVCT(): String {
        val vct = extendedAddressTAS.optString("vct")
        if (vct.isNullOrEmpty()) {
            throw Exception("Cannot find 'vct' field in extended Address TAS file")
        }
        return vct
    }

    fun getDiplomaVCT(): String {
        val vct = diplomaTAS.optString("vct")
        if (vct.isNullOrEmpty()) {
            throw Exception("Cannot find 'vct' field in diploma TAS file")
        }
        return vct
    }

    fun getInsuranceVCT(): String {
        val vct = insuranceTAS.optString("vct")
        if (vct.isNullOrEmpty()) {
            throw Exception("Cannot find 'vct' field in insurance TAS file")
        }
        return vct
    }

    fun getAddressDisplayName() = findDisplayName(extendedAddressTAS, baseAddressTAS)

    fun getDiplomaDisplayName() = findDisplayName(diplomaTAS)

    fun getInsuranceDisplayName() = findDisplayName(insuranceTAS)

    fun getAddressClaimLabel(pathValue: String): String {
        return findClaimLabel(extendedAddressTAS, baseAddressTAS, pathValue = pathValue)
    }

    fun getDiplomaClaimLabel(pathValue: String): String {
        return findClaimLabel(diplomaTAS, pathValue = pathValue)
    }

    fun getInsuranceClaimLabel(pathValue: String): String {
        return findClaimLabel(insuranceTAS, pathValue = pathValue)
    }

    private fun findDisplayName(vararg tasFiles: JSONObject): String {
        for (tas in tasFiles) {
            val displayName = findDisplayNameInTAS(tas)
            if (displayName != null) {
                return displayName
            }
        }
        throw Exception("Cannot find display name for language $language-$locale")
    }

    private fun findClaimLabel(vararg tasFiles: JSONObject, pathValue: String): String {
        for (tas in tasFiles) {
            val label = findClaimLabelInTAS(tas, pathValue)
            if (label != null) {
                return label
            }
        }
        throw Exception("Cannot find claim label for path: '$pathValue' and language $language-$locale in either TAS")
    }

    private fun findDisplayNameInTAS(tas: JSONObject): String? {
        val displayArray = tas.optJSONArray("display") ?: return null
        val display = findExactLanguageEntry(displayArray) ?: return null
        val name = display.optString("name")
        if (name.isNullOrEmpty()) {
            throw Exception("Display entry found but 'name' is missing for language $language-$locale")
        }
        return name
    }

    private fun findClaimLabelInTAS(tas: JSONObject, pathValue: String): String? {
        val claims = tas.optJSONArray("claims") ?: return null

        for (i in 0 until claims.length()) {
            val claim = claims.getJSONObject(i)
            val pathArray = claim.optJSONArray("path") ?: continue
            if (pathArray.length() == 1 && pathArray.getString(0) == pathValue) {
                val displayArray = claim.optJSONArray("display") ?: return null

                val display = findExactLanguageEntry(displayArray)
                return if (display != null) {
                    val label = display.optString("label")
                    if (!label.isNullOrEmpty()) {
                        label
                    } else {
                        throw Exception("Display entry for claim '$pathValue' is missing 'label' field in TAS")
                    }
                } else {
                    null
                }
            }
        }
        return null
    }

    private fun findExactLanguageEntry(displayArray: JSONArray): JSONObject? {
        for (i in 0 until displayArray.length()) {
            val display = displayArray.getJSONObject(i)
            if (display.optString("lang") == "$language-$locale") {
                return display
            }
        }
        return null
    }
}
