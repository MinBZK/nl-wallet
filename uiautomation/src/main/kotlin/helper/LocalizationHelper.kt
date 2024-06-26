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

    fun getString(key: String) = localizedStringsMap[language.lowercase()]?.get(key) as? String ?: ""

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

