package helper

import com.squareup.moshi.Moshi
import com.squareup.moshi.kotlin.reflect.KotlinJsonAdapterFactory
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


    fun translate(text: String): String {
        require(language in localizedStringsMap) { "Language '$language' is not supported" }
        if (language == "nl") {
            return text
        }
        check(language == "en") { "Unknown translation for language: $language" }
        return when (text) {
            "Persoonsgegevens" -> "Personal Data"
            "Woonadres" -> "Residential address"
            else -> throw IllegalArgumentException("No translation for: $text")
        }
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
        const val L10N_FILE_PATH = "../wallet_app/lib/l10n"
    }
}

