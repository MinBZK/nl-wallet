package nativesuite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    nativefeature.introduction.IntroductionTests::class,
    nativefeature.issuance.PidIssuanceTests::class,
    nativefeature.security.SetupRemotePinTests::class,
)
@Suite
@SuiteDisplayName("Full test suite")
object FullTestSuite
