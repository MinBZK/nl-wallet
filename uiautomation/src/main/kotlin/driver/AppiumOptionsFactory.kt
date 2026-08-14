package driver

import data.TestConfigRepository.Companion.testConfig
import io.appium.java_client.android.options.UiAutomator2Options
import io.appium.java_client.ios.options.XCUITestOptions
import util.EnvironmentUtil
import util.TestInfoHandler

internal const val APK_PATH = "../nl.ictu.edi.wallet.latest-0.6.0-release.apk"
internal const val IPA_PATH = "../nl.ictu.edi.wallet.latest-0.6.0.ipa"

internal fun buildAndroidOptions(appPath: String = ""): UiAutomator2Options {
    val autoGrant = EnvironmentUtil.getVar("AUTO_GRANT_PERMISSIONS").toBooleanStrictOrNull() ?: true
    val finalAppPath = appPath.ifBlank { testConfig.appPath }.ifBlank { APK_PATH }
    return UiAutomator2Options().apply {
        setApp(finalAppPath)
        setAppPackage(testConfig.appIdentifier)
        setLanguage(TestInfoHandler.language)
        setLocale(TestInfoHandler.locale)
        setIgnoreHiddenApiPolicyError(true)
        setCapability("appium:newCommandTimeout", 350)
        setCapability("appium:autoGrantPermissions", autoGrant)
        setCapability("appium:fullReset", true)
    }
}

internal fun buildIOSOptions(
    appPath: String = "",
    updatedWDABundleId: String,
    wdaLocalPort: Int? = null
): XCUITestOptions {
    val acceptAlerts = EnvironmentUtil.getVar("IOS_ACCEPT_ALERTS").toBooleanStrictOrNull() ?: true
    val finalAppPath = appPath.ifBlank { testConfig.appPath }.ifBlank { IPA_PATH }
    return XCUITestOptions().apply {
        setApp(finalAppPath)
        setBundleId(testConfig.appIdentifier)
        setLanguage(TestInfoHandler.language)
        setLocale(TestInfoHandler.locale)
        setCapability("appium:newCommandTimeout", 150)
        setCapability("appium:autoAcceptAlerts", acceptAlerts)
        setCapability("appium:showXcodeLog", true)
        setCapability("appium:includeSafariInWebviews", true)
        setCapability("appium:nativeWebTap", false)
        setCapability("appium:wdaLaunchTimeout", 60000)
        setCapability("appium:wdaConnectionTimeout", 180000)
        setCapability("appium:webkitResponseTimeout", 20000)
        setCapability("appium:webviewAtomWaitTimeout", 15000)
        setCapability("appium:boundElementsByIndex", true)
        setCapability("appium:customSnapshotTimeout", 15)
        setCapability("appium:settings[respectSystemAlerts]", true)
        setCapability("appium:xcodeOrgId", "XGL6UKBPLP")
        setCapability("appium:xcodeSigningId", "Apple Development")
        setCapability("appium:updatedWDABundleId", updatedWDABundleId)
        if (wdaLocalPort != null) setCapability("appium:wdaLocalPort", wdaLocalPort)
    }
}
