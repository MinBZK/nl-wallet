package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.menu_and_settings.ChangeLanguageTests::class,
    feature.menu_and_settings.ClearDataTests::class,
    feature.menu_and_settings.MenuTests::class,
    feature.menu_and_settings.HelpTests::class,
    )
@Suite
@SuiteDisplayName("Cards and history Test Suite")
object MenuAndSettingsTestSuite
