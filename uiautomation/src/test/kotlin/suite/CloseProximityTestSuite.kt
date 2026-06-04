package suite

import feature.close_proximity.CloseProximityDisclosureTests
import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    CloseProximityDisclosureTests::class,
)
@Suite
@SuiteDisplayName("Close Proximity Test Suite")
object CloseProximityTestSuite
