package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.introduction.IntroductionTests::class,
    feature.introduction.AppTourVideoTests::class,
    feature.issuance.PidIssuanceTests::class,
    feature.issuance.RenewCardTests::class,
    feature.issuance.DisclosureBasedIssuanceTests::class,
    feature.openapp.OpenAppTests::class,
    feature.security.SetupRemotePinTests::class,
)
@Suite
@SuiteDisplayName("Full test suite")
object FullTestSuite
