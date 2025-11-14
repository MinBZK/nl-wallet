package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.security.SetupRemotePinTests::class,
    feature.security.UserEntersPinTests::class,
    feature.security.UserLocksWalletTests::class,
    feature.security.RecoverPinTests::class,
)
@Suite
@SuiteDisplayName("Security Test Suite")
object SecurityTestSuite
