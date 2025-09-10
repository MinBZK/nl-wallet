package nativesuite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    nativefeature.issuance.PidIssuanceTests::class,
)
@Suite
@SuiteDisplayName("Personalize Test Suite")
object IssuanceTestSuite
