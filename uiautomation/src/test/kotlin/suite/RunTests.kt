package suite

import org.junit.platform.suite.api.SelectPackages
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectPackages("uiTests")
@Suite
@SuiteDisplayName("Run all tests")
object RunTests {
}
