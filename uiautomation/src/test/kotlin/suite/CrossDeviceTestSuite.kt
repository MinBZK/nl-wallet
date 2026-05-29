package suite

import feature.disclosure.CrossDeviceDisclosureTests
import feature.issuance.CrossDeviceDisclosureBasedIssuanceTests
import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    CrossDeviceDisclosureTests::class,
    CrossDeviceDisclosureBasedIssuanceTests::class,
)
@Suite
@SuiteDisplayName("Cross device test suite")
object CrossDeviceTestSuite
