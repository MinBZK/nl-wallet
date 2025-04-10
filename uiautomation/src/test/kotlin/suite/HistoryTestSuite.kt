package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.history.HistoryTests::class,
)
@Suite
@SuiteDisplayName("History Test Suite")
object HistoryTestSuite
