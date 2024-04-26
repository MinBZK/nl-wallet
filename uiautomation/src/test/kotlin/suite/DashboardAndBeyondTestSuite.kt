package suite

import org.junit.platform.suite.api.SelectPackages
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectPackages(
    "feature.card",
    "feature.confirm",
    "feature.dashboard",
    "feature.lock",
    "feature.menu",
    "feature.settings",
)
@Suite
@SuiteDisplayName("Dashboard and beyond Test Suite")
object DashboardAndBeyondTestSuite
