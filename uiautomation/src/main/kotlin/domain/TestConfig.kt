package domain

data class TestConfig(
    val appIdentifier: String, // package name (Android) or bundle ID (iOS)
    val deviceName: String,
    val platform: Platform,
    val platformVersion: String,
    val udid: String,
    val remote: Boolean,
    val automationName: String,
    val commitSha: String,
)
