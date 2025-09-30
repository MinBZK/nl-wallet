package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.cards_and_history.CardDetailTests::class,
    feature.cards_and_history.DashboardTests::class,
    feature.cards_and_history.HistoryTests::class,
)
@Suite
@SuiteDisplayName("Cards and history Test Suite")
object CardsAndHistoryTestSuite
