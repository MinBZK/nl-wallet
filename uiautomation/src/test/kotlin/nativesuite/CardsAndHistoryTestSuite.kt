package nativesuite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    nativefeature.cards_and_history.CardDetailTests::class,
    nativefeature.cards_and_history.DashboardTests::class,
    nativefeature.cards_and_history.HistoryTests::class,
)
@Suite
@SuiteDisplayName("Cards and history Test Suite")
object CardsAndHistoryTestSuite
