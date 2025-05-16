package helper

import helper.FileUtils.getProjectFile
import org.tomlj.Toml
import org.tomlj.TomlTable
import java.io.File


class IssuanceDataHelper {

    private val usecases: TomlTable

    init {
        val settings = envsubst(File(getProjectFile("scripts/devenv/demo_issuer.toml.template")).readText()) {
            if (it.endsWith("_PORT")) "0" else ""
        }
        usecases = Toml.parse(settings).getTable("usecases") ?: throw Exception("Use cases not found")
    }

    fun getAttributeValues(issuerType: String, bsn: String, attribute: String): List<String> {
        val docs = usecases.getArray(listOf(issuerType, bsn)) ?: return listOf()
        return (0..<docs.size()).map { index ->
            docs.getTable(index).get(listOf("attributes", attribute))?.toString() ?: throw Exception("Attribute not found: $attribute")
        }
    }
}
