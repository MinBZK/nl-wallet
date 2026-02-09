package helper

import com.squareup.moshi.Moshi
import com.squareup.moshi.kotlin.reflect.KotlinJsonAdapterFactory
import helper.FileUtils.getProjectFile
import util.TestInfoHandler.Companion.language
import java.io.File

class LocalizationHelper {
    private val localizedStringsMap = mutableMapOf<String, Map<*, *>>()

    init {
        loadLocalizedStrings()
    }

    fun getString(key: String): String = localizedStringsMap[language]?.let {
        it[key]?.toString() ?: throw IllegalArgumentException("Key $key does not exist in '$language'")
    } ?: throw IllegalArgumentException("Language '$language' is not configured")


    fun getPluralString(key: String, count: Int, placeholders: Map<String, String>): String {
        val template = getString(key)

        val pluralForms = parsePluralFormat(template)

        val selectedForm = if (count == 1) pluralForms["one"] else pluralForms["other"]
            ?: throw IllegalArgumentException("Plural form not found for count=$count in template: $template")

        var result = selectedForm
        placeholders.forEach { (placeholder, value) ->
            result = result?.replace("{$placeholder}", value)
        }
        return result.toString()
    }

    private fun parsePluralFormat(template: String): Map<String, String> {
        val pluralRegex = """\{[^,]+,\s*plural,\s*(.+)}""".toRegex()
        val match = pluralRegex.find(template)
            ?: throw IllegalArgumentException("Invalid plural format: $template")

        val formsContent = match.groupValues[1]
        val forms = mutableMapOf<String, String>()

        val formRegex = """\s*(\w+)\s*\{""".toRegex()
        var formPos = 0
        while (true) {
            val formMatch = formRegex.find(formsContent, formPos) ?: break
            val formName = formMatch.groupValues[1]
            val contentStart = formMatch.range.last + 1

            var braceCount = 1
            var pos = contentStart
            while (true) {
                when (formsContent[pos]) {
                    '{' -> braceCount++
                    '}' -> braceCount--
                }
                if (braceCount == 0) {
                    break
                }
                pos++
                if (pos == formsContent.length) {
                    throw IllegalStateException("unbalanced braces")
                }
            }

            forms[formName] = formsContent.substring(contentStart, pos)
            formPos = pos + 1
        }

        return forms
    }

    private fun loadLocalizedStrings() {
        val moshi = Moshi.Builder()
            .add(KotlinJsonAdapterFactory())
            .build()

        val adapter = moshi.adapter(Map::class.java)

        val languageList = listOf("en", "nl")
        for (language in languageList) {
            val jsonFile = File("$L10N_FILE_PATH/intl_$language.arb")
            localizedStringsMap[language] = adapter?.fromJson(jsonFile.readText()) as Map<*, *>
        }
    }

    companion object {
        val L10N_FILE_PATH = getProjectFile("wallet_app/lib/l10n")
    }
}
