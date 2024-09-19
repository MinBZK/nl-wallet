package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.appstart.AppStartTests::class,
)
@Suite
@SuiteDisplayName("AppStart Test Suite")
object AppStartTestSuite
