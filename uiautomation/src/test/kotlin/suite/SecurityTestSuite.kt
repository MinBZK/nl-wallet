package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.security.SecurityChoosePinTests::class,
    feature.security.SecurityConfirmPinTests::class,
    feature.security.SecuritySetupCompletedTests::class,
)
@Suite
@SuiteDisplayName("Security Test Suite")
object SecurityTestSuite
