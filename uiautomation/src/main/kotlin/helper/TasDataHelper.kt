package helper

import domain.Platform
import helper.FileUtils.getProjectFile
import org.json.JSONArray
import org.json.JSONObject
import util.TestInfoHandler.Companion.language
import util.TestInfoHandler.Companion.locale
import util.TestInfoHandler.Companion.platform
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

    private val diplomaTAS: JSONObject by lazy {
        val jsonContent = File(getDiplomaCardMetadataPath()).readText(Charsets.UTF_8)
        JSONObject(jsonContent)
    }

    private val insuranceTAS: JSONObject by lazy {
        val jsonContent = File(getInsuranceCardMetadataPath()).readText(Charsets.UTF_8)
        JSONObject(jsonContent)
    }

    private val museumMaandkaartTAS: JSONObject by lazy {
        val jsonContent = File(getMuseumMaandkaartCardMetadataPath()).readText(Charsets.UTF_8)
        JSONObject(jsonContent)
    }

    private val drivingLicenseTAS: JSONObject by lazy {
        val jsonContent = File(getDrivingLicenseMetadataPath()).readText(Charsets.UTF_8)
        JSONObject(jsonContent)
    }

    private val registrationCertificateTAS: JSONObject by lazy {
        val jsonContent = File(getRegistrationCertificateMetadataPath()).readText(Charsets.UTF_8)
        JSONObject(jsonContent)
    }

    private val loyaltyTAS: JSONObject by lazy {
        val jsonContent = File(getLoyaltyCardMetadataPath()).readText(Charsets.UTF_8)
        JSONObject(jsonContent)
    }

    // Pid functions

    fun getPidVCT(): String {
        val vct = extendedPidTAS.optString("vct")
        if (vct.isNullOrEmpty()) {
            throw Exception("Cannot find 'vct' field in extended PID TAS file")
        }
        return vct
    }

    fun getPidDisplayName() = findDisplayName(extendedPidTAS, basePidTAS)

    fun getPidClaimLabel(vararg pathValue: String): String {
        return findClaimLabel(extendedPidTAS, basePidTAS, pathValue = pathValue)
    }

    private fun getExtendedPidCardMetadataPath() = getProjectFile("scripts/devenv/eudi_pid_nl_1.json")

    private fun getBasePidCardMetadataPath() = getProjectFile("scripts/devenv/eudi_pid_1.json")

    //Diploma functions
    fun getDiplomaVCT(): String {
        val vct = diplomaTAS.optString("vct")
        if (vct.isNullOrEmpty()) {
            throw Exception("Cannot find 'vct' field in diploma TAS file")
        }
        return vct
    }

    fun getDiplomaDisplayName() = findDisplayName(diplomaTAS)

    fun getDiplomaClaimLabel(vararg pathValue: String): String {
        return findClaimLabel(diplomaTAS, pathValue = pathValue)
    }

    private fun getDiplomaCardMetadataPath() = getProjectFile("scripts/devenv/com.example.degree.json")

    //Insurance functions
    fun getInsuranceVCT(): String {
        val vct = insuranceTAS.optString("vct")
        if (vct.isNullOrEmpty()) {
            throw Exception("Cannot find 'vct' field in insurance TAS file")
        }
        return vct
    }

    fun getInsuranceDisplayName() = findDisplayName(insuranceTAS)

    fun getInsuranceClaimLabel(vararg pathValue: String): String {
        return findClaimLabel(insuranceTAS, pathValue = pathValue)
    }

    private fun getInsuranceCardMetadataPath() = getProjectFile("scripts/devenv/com.example.insurance.mdoc.json")

    //Loyalty functions
    fun getLoyaltyDisplayName() = findDisplayName(loyaltyTAS)

    private fun getLoyaltyCardMetadataPath() = getProjectFile("scripts/devenv/com.example.jum.bonuskaart.sd_jwt.json")

    //Museum Maandkaart functions

    fun getMuseumMaandkaartDisplayName() = findDisplayName(museumMaandkaartTAS)

    private fun getMuseumMaandkaartCardMetadataPath() =
        getProjectFile("scripts/devenv/com.example.museum_maandkaart.mdoc.json")

    //Driving License functions and values
    fun getDrivingLicenseDisplayName() = findDisplayName(drivingLicenseTAS)

    fun getDrivingLicenseClaimLabel(vararg pathValue: String): String {
        return findClaimLabel(drivingLicenseTAS, pathValue = pathValue)
    }

    private fun getDrivingLicenseMetadataPath() =
        getProjectFile("scripts/devenv/org.iso.18013.5.1.mDL.mdoc.json")

    //Registration certificate functions
    fun getRegistrationCertificateDisplayName() = findDisplayName(registrationCertificateTAS)

    fun getRegistrationCertificateClaimLabel(vararg pathValue: String): String {
        return findClaimLabel(registrationCertificateTAS, pathValue = pathValue)
    }

    private fun getRegistrationCertificateMetadataPath() =
        getProjectFile("scripts/devenv/org.iso.7367.2.1.mVC.mdoc.json")

    //Generic functions for handling TAS files, to be used for all cards.
    private fun findDisplayName(vararg tasFiles: JSONObject): String {
        for (tas in tasFiles) {
            val displayName = findDisplayNameInTAS(tas)
            if (displayName != null) {
                return displayName
            }
        }
        throw Exception("Cannot find display name for language/locale $language-$locale")
    }

    private fun findClaimLabel(vararg tasFiles: JSONObject, pathValue: Array<out String>): String {
        for (tas in tasFiles) {
            val label = findClaimLabelInTAS(tas, pathValue)
            if (label != null) {
                return label
            }
        }
        throw Exception(
            "Cannot find claim label for path: '${pathValue.joinToString(".")}' and language $language-$locale in either TAS"
        )
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

    private fun findClaimLabelInTAS(tas: JSONObject, pathValue: Array<out String>): String? {
        val claims = tas.optJSONArray("claims") ?: return null

        for (i in 0 until claims.length()) {
            val claim = claims.getJSONObject(i)
            val pathArray = claim.optJSONArray("path") ?: continue
            if (pathEndsWith(pathArray, pathValue)) {
                val displayArray = claim.optJSONArray("display") ?: return null

                val display = findExactLanguageEntry(displayArray)
                return if (display != null) {
                    // SD-JWT type metadata labels its claims with 'label', mdoc issuer metadata with 'name'.
                    val label = display.optString("label").ifEmpty { display.optString("name") }
                    if (label.isNotEmpty()) {
                        label
                    } else {
                        throw Exception(
                            "Display entry for claim '${pathValue.joinToString(".")}' is missing 'label'/'name' field in TAS"
                        )
                    }
                } else {
                    null
                }
            }
        }
        return null
    }

    /**
     * Matches the trailing segments of a claim path, so that namespaced mdoc paths such as
     * ["org.iso.18013.5.1", "family_name"] can be addressed by name only.
     */
    private fun pathEndsWith(claimPath: JSONArray, path: Array<out String>): Boolean {
        if (path.isEmpty() || claimPath.length() < path.size) {
            return false
        }
        val offset = claimPath.length() - path.size
        return path.withIndex().all { (index, segment) -> claimPath.optString(offset + index) == segment }
    }

    private fun findExactLanguageEntry(displayArray: JSONArray): JSONObject? {
        for (i in 0 until displayArray.length()) {
            val display = displayArray.getJSONObject(i)

            when (platform) {
                Platform.ANDROID -> {
                    if (display.optString("locale") == "$language-$locale") {
                        return display
                    }
                }

                Platform.IOS -> {
                    // The iOS locale capability uses an underscore for English (en_US) while TAS files use a hyphen.
                    if (display.optString("locale") == locale.replace("_", "-")) {
                        return display
                    }
                }
            }
        }
        return null
    }
}
