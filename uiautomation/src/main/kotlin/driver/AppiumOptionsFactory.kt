package driver

import data.TestConfigRepository.Companion.testConfig
import io.appium.java_client.android.options.UiAutomator2Options
import io.appium.java_client.ios.options.XCUITestOptions
import util.EnvironmentUtil
import util.TestInfoHandler

internal const val APK_PATH = "../nl.ictu.edi.wallet.latest-0.6.0-release.apk"
internal const val IPA_PATH = "../nl.ictu.edi.wallet.latest-0.6.0.ipa"

internal fun buildAndroidOptions(): UiAutomator2Options {
    val autoGrant = EnvironmentUtil.getVar("AUTO_GRANT_PERMISSIONS").toBooleanStrictOrNull() ?: true
    return UiAutomator2Options().apply {
        setApp(APK_PATH)
        setAppPackage(testConfig.appIdentifier)
        setLanguage(TestInfoHandler.language)
        setLocale(TestInfoHandler.locale)
        setIgnoreHiddenApiPolicyError(true)
        setCapability("appium:newCommandTimeout", 350)
        setCapability("appium:autoGrantPermissions", autoGrant)
        setCapability("appium:fullReset", true)
    }
}

internal fun buildIOSOptions(updatedWDABundleId: String, wdaLocalPort: Int? = null): XCUITestOptions {
    val acceptAlerts = EnvironmentUtil.getVar("IOS_ACCEPT_ALERTS").toBooleanStrictOrNull() ?: true
    return XCUITestOptions().apply {
        setApp(IPA_PATH)
        setBundleId(testConfig.appIdentifier)
        setLanguage(TestInfoHandler.language)
        setLocale(TestInfoHandler.locale)
        setCapability("appium:newCommandTimeout", 350)
        setCapability("appium:autoAcceptAlerts", acceptAlerts)
        setCapability("appium:showXcodeLog", true)
        setCapability("appium:includeSafariInWebviews", true)
        setCapability("appium:nativeWebTap", true)
        setCapability("appium:wdaLaunchTimeout", 60000)
        setCapability("appium:wdaConnectionTimeout", 60000)
        setCapability("appium:webkitResponseTimeout", 20000)
        setCapability("appium:xcodeOrgId", "XGL6UKBPLP")
        setCapability("appium:xcodeSigningId", "iPhone Developer")
        setCapability("appium:updatedWDABundleId", updatedWDABundleId)
        if (wdaLocalPort != null) setCapability("appium:wdaLocalPort", wdaLocalPort)
    }
}
