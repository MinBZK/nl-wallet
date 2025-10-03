package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.openapp.OpenAppTests::class,
)
@Suite
@SuiteDisplayName("Open app Test Suite")
object OpenAppTestSuite
