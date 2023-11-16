package suite

import org.junit.platform.suite.api.SelectPackages
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectPackages("feature")
@Suite
@SuiteDisplayName("Feature test suite")
object FeatureTestSuite
