package suite

import org.junit.platform.suite.api.SelectPackages
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectPackages(
    "feature.appstart",
    "feature.introduction",
    "feature.security",
    "feature.personalize",
    "feature.web",
)
@Suite
@SuiteDisplayName("App start to dashboard Test Suite")
object AppStartToDashboardTestSuite
