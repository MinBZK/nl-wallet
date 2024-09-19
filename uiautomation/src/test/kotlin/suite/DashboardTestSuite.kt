package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.dashboard.DashboardTests::class,
)
@Suite
@SuiteDisplayName("Dashboard Test Suite")
object DashboardTestSuite
