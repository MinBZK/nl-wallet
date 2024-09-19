package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.settings.ChangeLanguageTests::class,
    feature.settings.ClearDataTests::class,
)
@Suite
@SuiteDisplayName("Settings Test Suite")
object SettingsTestSuite
