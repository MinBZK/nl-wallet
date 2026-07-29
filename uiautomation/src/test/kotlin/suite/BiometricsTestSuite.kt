package suite

import feature.security.BiometricsTests
import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    BiometricsTests::class,
)
@Suite
@SuiteDisplayName("Biometrics Test Suite")
object BiometricsTestSuite
