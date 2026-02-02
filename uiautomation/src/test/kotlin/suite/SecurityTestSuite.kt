package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.security.SetupSecurityTests::class,
    feature.security.UserEntersPinTests::class,
    feature.security.UserLocksWalletTests::class,
    feature.security.RecoverPinTests::class,
    feature.security.ChangeRemotePinTests::class,
    )
@Suite
@SuiteDisplayName("Security Test Suite")
object SecurityTestSuite
