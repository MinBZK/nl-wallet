package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.issuance.DisclosureBasedIssuanceTests::class,
    feature.issuance.GenericIssuanceTests::class,
)
@Suite
@SuiteDisplayName("Card Issuance Test Suite")
object CardIssuanceTestSuite
