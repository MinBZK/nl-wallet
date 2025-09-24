package nativesuite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    nativefeature.introduction.IntroductionTests::class,
    nativefeature.introduction.AppTourVideoTests::class,
)
@Suite
@SuiteDisplayName("Introduction Test Suite")
object IntroductionTestSuite
