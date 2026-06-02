package helper

import com.upokecenter.cbor.CBORObject
import com.upokecenter.cbor.CBORType

data class MdocAttribute(val namespace: String, val identifier: String, val value: String)
data class MdocDocument(val docType: String, val attributes: List<MdocAttribute>)
data class ParsedDeviceResponse(val version: String, val status: Int, val documents: List<MdocDocument>)

object DeviceResponseHelper {

    private const val HEX_MARKER = "CLOSE_PROXIMITY_DEVICE_RESPONSE_HEX="

    fun extractHex(output: String): String? =
        output.lines()
            .firstOrNull { it.startsWith(HEX_MARKER) }
            ?.removePrefix(HEX_MARKER)
            ?.trim()

    fun parse(hex: String): ParsedDeviceResponse {
        val bytes = hex.chunked(2).map { it.toInt(16).toByte() }.toByteArray()
        val root = CBORObject.DecodeFromBytes(bytes)
        val version = root["version"].AsString()
        val status = root["status"].AsInt32()
        val documents = root["documents"]?.values?.map { doc ->
            val docType = doc["docType"].AsString()
            val nameSpaces = doc["issuerSigned"]["nameSpaces"]
            val attributes = mutableListOf<MdocAttribute>()
            for (ns in nameSpaces.keys) {
                val nsStr = ns.AsString()
                for (itemBytes in nameSpaces[ns].values) {
                    // Each item is CBOR tag 24: an embedded CBOR-encoded IssuerSignedItem
                    val item = CBORObject.DecodeFromBytes(itemBytes.GetByteString())
                    val identifier = item["elementIdentifier"].AsString()
                    val value = cborToString(item["elementValue"])
                    attributes += MdocAttribute(nsStr, identifier, value)
                }
            }
            MdocDocument(docType, attributes)
        } ?: emptyList()
        return ParsedDeviceResponse(version, status, documents)
    }

    private fun cborToString(cbor: CBORObject): String = when (cbor.type) {
        CBORType.TextString -> cbor.AsString()
        CBORType.Integer -> cbor.AsInt64Value().toString()
        CBORType.Boolean -> cbor.isTrue.toString()
        else -> cbor.toString()
    }
}
