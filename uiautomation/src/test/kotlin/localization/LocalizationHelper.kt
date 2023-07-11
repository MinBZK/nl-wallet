package localization

import com.squareup.moshi.JsonAdapter
import com.squareup.moshi.Moshi
import com.squareup.moshi.kotlin.reflect.KotlinJsonAdapterFactory
import util.SetupTestTagHandler.Companion.language
import java.io.File

class LocalizationHelper {

    fun getLocalizedString(key: String): Any? {

        val path = "../wallet_app/lib/l10n"
        val jsonFile = if (language == "NL") {
            File("$path/intl_nl.arb")
        } else {
            File("$path/intl_en.arb")
        }
        val json = jsonFile.readText()

        val moshi = Moshi.Builder().add(KotlinJsonAdapterFactory()).build()
        val adapter: JsonAdapter<Map<*, *>>? = moshi.adapter(Map::class.java)

        val localizedStrings = adapter?.fromJson(json)
        return localizedStrings?.get(key)
    }
}

