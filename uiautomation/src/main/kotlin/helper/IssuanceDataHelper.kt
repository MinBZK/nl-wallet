package helper

import helper.FileUtils.getProjectFile
import org.json.JSONArray
import org.json.JSONObject
import java.io.File

class IssuanceDataHelper {

    private val useCases: JSONObject
    private val l10n = LocalizationHelper()

    init {
        val jsonTemplate = File(getProjectFile("scripts/devenv/demo_issuer.json.template")).readText()
        val settings = envsubst(jsonTemplate) {
            if (it.endsWith("_PORT")) "0" else ""
        }
        val root = JSONObject(settings)
        useCases = root.optJSONObject("usecases")
            ?: throw Exception("Use cases not found")
    }

    fun getAttributeValues(issuerType: String, bsn: String, attribute: String): List<String> {
        return candidateUseCaseKeys(issuerType).flatMap { key ->
            val docs = useCases.optJSONObject(key)?.optJSONObject("data")?.optJSONArray(bsn)
                ?: return@flatMap emptyList()
            collectAttributeValues(docs, attribute)
        }
    }

    private fun candidateUseCaseKeys(issuerType: String): List<String> =
        useCases.keys().asSequence()
            .filter { it == issuerType || it.startsWith("${issuerType}_") }
            .toList()

    private fun collectAttributeValues(docsForBsn: JSONArray, attribute: String): List<String> {
        return docsForBsn.mapNotNull { doc ->
            if (doc !is JSONObject) {
                return@mapNotNull null
            }
            val attrs = doc.optJSONObject("attributes")
                ?: throw Exception("Attribute object not found for doc $doc")

            val value = attrs.opt(attribute)
                ?: throw Exception("Attribute not found: $attribute")
            jsonValueToString(value)
        }
    }

    private fun jsonValueToString(value: Any): String {
        if (value === JSONObject.NULL) return l10n.getString("cardValueNull")
        return value.toString()
    }
}
