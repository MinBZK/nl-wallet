package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.revocation.RevokeCardTests::class,
    feature.revocation.ViewRevocationCodeTests::class,
    )
@Suite
@SuiteDisplayName("Revocation Test Suite")
object RevocationTestSuite
