package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.web.digid.MockDigidWebTests::class,
    feature.web.rp.RelyingPartyWebTests::class,
)
@Suite
@SuiteDisplayName("Web Test Suite")
object WebTestSuite
