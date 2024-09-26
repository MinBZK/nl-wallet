package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.confirm.UserEntersPinTests::class,
    feature.confirm.UserForgetsPinTests::class,
)
@Suite
@SuiteDisplayName("Confirm Test Suite")
object ConfirmTestSuite
