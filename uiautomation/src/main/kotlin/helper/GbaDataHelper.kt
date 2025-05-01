package helper

import helper.FileUtils.getProjectFile
import org.w3c.dom.Document
import org.w3c.dom.Node
import java.io.File
import javax.xml.parsers.DocumentBuilderFactory

class GbaDataHelper {

    private val gbaDataDirectory = getProjectFile("wallet_core/gba_hc_converter/resources/gba-v-responses")

    enum class Field(val nummer: String) {
        FIRST_NAME("210"),
        NAME("240"),
        BIRTH_DATE("310"),
        STREET("1110"),
        HOUSE_NUMBER("1120"),
        POSTAL_CODE("1160"),
        CITY("1170")
    }

    private fun loadXmlDocument(fileName: String): Document {
        val file = File("$gbaDataDirectory/$fileName.xml")
        val factory = DocumentBuilderFactory.newInstance().apply {
            isNamespaceAware = true
        }
        val builder = factory.newDocumentBuilder()
        return builder.parse(file)
    }

    fun getValueByField(field: Field, bsn: String): String {
        val targetNummer = field.nummer
        val doc = loadXmlDocument(bsn)
        val elements = doc.getElementsByTagNameNS("*", "item")

        for (i in 0 until elements.length) {
            val itemNode = elements.item(i)
            if (itemNode.nodeType == Node.ELEMENT_NODE && itemNode.parentNode.nodeName.endsWith("elementen")) {
                var number: String? = null
                var value: String? = null

                val children = itemNode.childNodes
                for (j in 0 until children.length) {
                    val child = children.item(j)
                    when {
                        child.nodeName.endsWith("nummer") -> number = child.textContent.trim()
                        child.nodeName.endsWith("waarde") -> value = child.textContent.trim()
                    }
                }
                if (number == targetNummer) {
                    return value ?: throw IllegalArgumentException("Waarde not found for nummer $number")
                }
            }
        }
        throw Exception("Cannot find attribute with nummer $targetNummer")
    }
}

