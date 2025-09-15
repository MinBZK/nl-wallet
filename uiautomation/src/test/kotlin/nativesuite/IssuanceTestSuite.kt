package nativesuite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    nativefeature.issuance.PidIssuanceTests::class,
    nativefeature.issuance.DisclosureBasedIssuanceTests::class,
    nativefeature.issuance.RenewCardTests::class
)
@Suite
@SuiteDisplayName("Issuance Test Suite")
object IssuanceTestSuite
