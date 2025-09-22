package nativesuite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    nativefeature.disclosure.DisclosureTests::class,
    nativefeature.disclosure.UniversalLinkTests::class,
    nativefeature.disclosure.QRScannerTests::class,
)
@Suite
@SuiteDisplayName("Disclosure Test Suite")
object DisclosureTestSuite
