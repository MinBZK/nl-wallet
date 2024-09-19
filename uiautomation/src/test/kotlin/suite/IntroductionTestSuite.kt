package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.introduction.IntroductionTests::class,
    feature.introduction.IntroductionPrivacyTests::class,
    feature.introduction.IntroductionConditionsTests::class,
)
@Suite
@SuiteDisplayName("Introduction Test Suite")
object IntroductionTestSuite
