package domain

enum class Platform {
    ANDROID, IOS;

    companion object {
        fun fromString(value: String): Platform = when (value.uppercase()) {
            "ANDROID" -> ANDROID
            "IOS" -> IOS
            else -> throw IllegalArgumentException("Unsupported platform: $value")
        }
    }
}
