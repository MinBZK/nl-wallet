package nativesuite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    nativefeature.security.SetupRemotePinTests::class,
    nativefeature.security.UserEntersPinTests::class,
    nativefeature.security.UserLocksWalletTests::class,
)
@Suite
@SuiteDisplayName("Security Test Suite")
object SecurityTestSuite
