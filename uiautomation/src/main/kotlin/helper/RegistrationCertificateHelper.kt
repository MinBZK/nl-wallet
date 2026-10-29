package helper

import org.json.JSONArray
import org.json.JSONObject
import util.EnvironmentUtil
import util.TestInfoHandler.Companion.language
import java.io.File
import java.net.URI
import java.util.Locale

class RegistrationCertificateHelper(private val fileEnvVar: String = XYZ_BANK_FILE_VAR) {

    companion object {
        const val XYZ_BANK_FILE_VAR = "XYZ_BANK_WRPRC_FILE"
    }

    private val payload: JSONObject by lazy {
        val path = EnvironmentUtil.getVar(fileEnvVar)
        if (path.isEmpty()) {
            throw IllegalStateException(
                "Environment variable $fileEnvVar is not set."
            )
        }
        val file = File(path)
        if (!file.exists()) {
            throw IllegalStateException("Registration certificate payload '$path' ($fileEnvVar) does not exist")
        }
        JSONObject(file.readText())
    }
    fun displayName(): String = payload.getString("name")

    fun legalName(): String = payload.getString("sub_ln")

    fun organizationId(): String = payload.getString("sub")

    fun countryCode(): String = payload.getString("country")

    fun countryName(): String = Locale.of("", countryCode()).getDisplayCountry(Locale.of(language))

    fun publicBody(): Boolean? = if (payload.isNull("public_body")) null else payload.getBoolean("public_body")

    fun websiteUri(): String? = optionalUri("info_uri")

    fun privacyPolicyUri(): String? = optionalUri("privacy_policy")

    fun supportUri(): String? {
        val supportUri = optionalUri("support_uri")?.trim() ?: return null
        val hasScheme = runCatching { URI(supportUri).scheme }.getOrNull() != null
        return when {
            hasScheme -> supportUri
            supportUri.contains("@") -> "mailto:$supportUri"
            else -> null
        }
    }

    // The sharing reason shown during disclosure; the app resolves the active language and falls back to English.
    fun purpose(): String? {
        val translations = linkedMapOf<String, String>()
        val purpose = payload.optJSONArray("purpose") ?: return null
        purpose.forEach { translation ->
            (translation as JSONObject).let { translations[it.getString("lang")] = it.getString("value") }
        }
        return translations[language] ?: translations["en"] ?: translations.values.firstOrNull()
    }

    fun description(): String? {
        val translations = linkedMapOf<String, MutableList<String>>()
        val descriptions = payload.optJSONArray("srv_description") ?: return null
        descriptions.forEach { description ->
            (description as JSONArray).forEach { translation ->
                (translation as JSONObject).let {
                    translations.getOrPut(it.getString("lang")) { mutableListOf() }.add(it.getString("value"))
                }
            }
        }
        val values = translations[language] ?: translations["en"] ?: translations.values.firstOrNull()
        return values?.joinToString("\n")
    }

    private fun optionalUri(claim: String): String? = if (payload.isNull(claim)) null else payload.getString(claim)
}
