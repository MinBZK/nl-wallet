package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.card.CardDetailTests::class,
    feature.card.CardDataTests::class,
)
@Suite
@SuiteDisplayName("Card Test Suite")
object CardTestSuite
