package domain

data class TestConfig(
    val appIdentifier: String, // package name (Android) or bundle ID (iOS)
    val deviceName: String,
    val platformName: String,
    val platformVersion: String,
    val remote: Boolean,
)
