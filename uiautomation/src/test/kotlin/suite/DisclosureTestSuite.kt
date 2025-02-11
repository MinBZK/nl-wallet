package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.disclosure.DisclosureTests::class,
)
@Suite
@SuiteDisplayName("Disclosure Test Suite")
object DisclosureTestSuite