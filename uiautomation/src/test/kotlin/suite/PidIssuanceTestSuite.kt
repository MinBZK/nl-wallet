package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.issuance.PidIssuanceTests::class,
    feature.issuance.RenewCardTests::class,
)
@Suite
@SuiteDisplayName("PID Issuance Test Suite")
object PidIssuanceTestSuite
