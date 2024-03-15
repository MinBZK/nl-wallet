package feature.card

import helper.TestBase
import navigator.CardNavigator
import navigator.screen.CardScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junitpioneer.jupiter.RetryingTest
import screen.card.CardDataIncorrectScreen
import screen.card.CardDataScreen
import screen.card.CardDetailScreen

@DisplayName("UC 7.2 - App shows all card attributes [PVW-1229]")
class CardDataTests : TestBase() {

    private lateinit var cardDataScreen: CardDataScreen

    @BeforeEach
    fun setUp() {
        CardNavigator().toScreen(CardScreen.CardData)

        cardDataScreen = CardDataScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("1. The Card attributes page displays all attributes on the card.")
    fun verifyCardData() {
        assertTrue(cardDataScreen.visible(), "card data screen is not visible")
        assertTrue(cardDataScreen.dataAttributesVisible(), "data attributes are not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("2. The User can go back to the Card detail page.")
    fun verifyBackButton() {
        cardDataScreen.clickBottomBackButton()

        val cardDetailScreen = CardDetailScreen()
        assertTrue(cardDetailScreen.visible(), "card detail screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("3. The App displays a warning indicating that the shown data is only for the User to see.")
    fun verifyDataPrivacyBanner() {
        assertTrue(cardDataScreen.dataPrivacyBannerVisible(), "data privacy banner not visible")
    }

    //This test is commented out because of a missing UI element and click behaviour, bug ticket link: https://SSSS/browse/PVW-2401
    /*@RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("4. Clicking the warning provides: A motivation why it is important to keep data to yourself AND An explanation how the User can share this data (with QR).")
    fun verifyDataPrivacySheet() {
        cardDataScreen.clickDataPrivacyBanner()

        assertTrue(cardDataScreen.dataPrivacySheetVisible(), "data privacy sheet not visible")
    }*/

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @Tags(Tag("english"))
    @DisplayName("5. The Card attribute labels are multi-lingual.")
    fun verifyDataLabelMultiLingual() {
        assertTrue(cardDataScreen.englishDataLabelsVisible(), "english data labels are not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @Tags(Tag("english"))
    @DisplayName("6. The Card attribute values are multi-lingual if applicable and are rendered according to their schema.")
    fun verifyDataValueMultiLingual() {
        assertTrue(cardDataScreen.englishDataValuesVisible(), "english data values are not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("7. The App provides a button for help if the data seems incorrect. This leads to a help screen explaining what to do in case of incorrect data.")
    fun verifyDataIncorrectButton() {
        cardDataScreen.clickDataIncorrectButton()

        val cardDataIncorrectScreen = CardDataIncorrectScreen()
        assertTrue(cardDataIncorrectScreen.visible(), "card data incorrect screen is not visible")
    }
}
