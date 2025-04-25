package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.personalize.PersonalizeInformTests::class,
    feature.personalize.PersonalizeAuthenticatingWithDigidScreenTests::class,
    feature.personalize.PersonalizePidPreviewTests::class,
    feature.personalize.PersonalizeSuccessTests::class,
    feature.personalize.PersonalizePidDataIncorrectTests::class,
    feature.personalize.PersonalizeAppHandlesDigidAuthenticationTests::class,
)
@Suite
@SuiteDisplayName("Personalize Test Suite")
object PersonalizeTestSuite
