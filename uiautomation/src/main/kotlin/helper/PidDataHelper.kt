package helper

import helper.FileUtils.getProjectFile
import org.json.JSONArray
import org.json.JSONObject
import java.io.File

class PidDataHelper {

    private val personAttributesFile = getProjectFile("wallet_core/wallet_server/pid_issuer/persons.json")

    enum class Field(val path: List<String>) {
        FIRST_NAME(listOf("given_name")),
        NAME(listOf("family_name")),
        STREET(listOf("address", "street")),
        HOUSE_NUMBER(listOf("address", "house_number")),
        POSTAL_CODE(listOf("address", "postal_code")),
        CITY(listOf("address", "city")),
    }

    fun getValueByField(field: Field, bsn: String): String {
        val persons = JSONArray(File(personAttributesFile).readText())
        val person = persons.map{ it as JSONObject }.find { it.getString("bsn") == bsn }
            ?: throw IllegalArgumentException("Person with bsn $bsn not found")

        return field.path.subList(0, field.path.size - 1).fold(person) { obj, item ->
            obj.getJSONObject(item)
        }.getString(field.path.last())
    }
}

