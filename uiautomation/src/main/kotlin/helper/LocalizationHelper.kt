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

    enum class Translation(
        val nl: String,
        val en: String,
    ) {
        SECONDS("seconden", "seconds"),
        ADD_CARD("Voeg kaart toe", "Add 1 card"),
    }

    fun translate(translation: Translation): String {
        return when (language) {
            "nl" -> translation.nl
            "en" -> translation.en
            else -> throw IllegalArgumentException("Language `$language` is not supported")
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
        val L10N_FILE_PATH = getProjectFile("wallet_app/lib/l10n")
    }
}

