package nativesuite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    nativefeature.menu_and_settings.ChangeLanguageTests::class,
    nativefeature.menu_and_settings.ClearDataTests::class,
    nativefeature.menu_and_settings.MenuTests::class,
)
@Suite
@SuiteDisplayName("Cards and history Test Suite")
object MenuAndSettingsTestSuite
