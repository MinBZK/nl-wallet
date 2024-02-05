package helper

import com.squareup.moshi.JsonAdapter
import com.squareup.moshi.Moshi
import com.squareup.moshi.kotlin.reflect.KotlinJsonAdapterFactory
import util.SetupTestTagHandler.Companion.language
import java.io.File

class LocalizationHelper {

    fun getString(key: String): String {
        val path = "../wallet_app/lib/l10n"
        val jsonFile = File("$path/intl_`${if (language == "NL") "nl" else "en"}`.arb")
        val json = jsonFile.readText()

        val moshi = Moshi.Builder().add(KotlinJsonAdapterFactory()).build()
        val adapter: JsonAdapter<Map<*, *>>? = moshi.adapter(Map::class.java)

        val localizedStrings = adapter?.fromJson(json)
        return localizedStrings?.get(key) as? String ?: ""
    }
}

