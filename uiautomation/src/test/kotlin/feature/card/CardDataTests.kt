package feature.card

import helper.TestBase
import navigator.CardNavigator
import navigator.screen.CardScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.card.CardDataIncorrectScreen
import screen.card.CardDataScreen
import screen.card.CardDetailScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${CardDataTests.USE_CASE} App shows all card attributes [${CardDataTests.JIRA_ID}]")
class CardDataTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 7.2"
        const val JIRA_ID = "PVW-1229"
    }

    private lateinit var cardDataScreen: CardDataScreen

    @BeforeEach
    fun setUp() {
        CardNavigator().toScreen(CardScreen.CardData)

        cardDataScreen = CardDataScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 The Card attributes page displays all attributes on the card. [${JIRA_ID}]")
    fun verifyCardData() {
        assertTrue(cardDataScreen.visible(), "card data screen is not visible")
        assertTrue(cardDataScreen.dataAttributesVisible(), "data attributes are not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.2 The User can go back to the Card detail page. [${JIRA_ID}]")
    fun verifyBackButton() {
        cardDataScreen.clickBottomBackButton()

        val cardDetailScreen = CardDetailScreen()
        assertTrue(cardDetailScreen.visible(), "card detail screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.3 The App displays a warning indicating that the shown data is only for the User to see. [${JIRA_ID}]")
    fun verifyDataPrivacyBanner() {
        assertTrue(cardDataScreen.dataPrivacyBannerVisible(), "data privacy banner not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @Tags(Tag("english"))
    @DisplayName("$USE_CASE.4 The Card attribute labels are multi-lingual. [${JIRA_ID}]")
    fun verifyDataLabelMultiLingual() {
        assertTrue(cardDataScreen.englishDataLabelsVisible(), "english data labels are not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @Tags(Tag("english"))
    @DisplayName("$USE_CASE.5 The Card attribute values are multi-lingual if applicable and are rendered according to their schema. [${JIRA_ID}]")
    fun verifyDataValueMultiLingual() {
        assertTrue(cardDataScreen.englishDataValuesVisible(), "english data values are not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.6 The App provides a button for help if the data seems incorrect. This leads to a help screen explaining what to do in case of incorrect data. [${JIRA_ID}]")
    fun verifyDataIncorrectButton() {
        cardDataScreen.scrollToEnd()
        cardDataScreen.clickDataIncorrectButton()

        val cardDataIncorrectScreen = CardDataIncorrectScreen()
        assertTrue(cardDataIncorrectScreen.visible(), "card data incorrect screen is not visible")
    }
}
