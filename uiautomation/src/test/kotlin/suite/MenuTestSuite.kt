package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.menu.MenuTests::class,
)
@Suite
@SuiteDisplayName("Menu Test Suite")
object MenuTestSuite
