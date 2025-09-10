package nativesuite

import nativefeature.introduction.IntroductionTests
import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    IntroductionTests::class,
//    feature.introduction.AppTourVideoTests::class,
)
@Suite
@SuiteDisplayName("Introduction Test Suite")
object IntroductionTestSuite
