package nl.rijksoverheid.edi.wallet.platform_support.util

fun List<UByte>.toByteList(): List<Byte> = this.map { it.toByte() }

fun List<UByte>.toByteArray(): ByteArray = this.toByteList().toByteArray()

fun ByteArray.toUByteList(): List<UByte> = this.map { it.toUByte() }