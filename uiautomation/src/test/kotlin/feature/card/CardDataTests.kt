package feature.card

import helper.GbaDataHelper
import helper.GbaDataHelper.Field.FIRST_NAME
import helper.GbaDataHelper.Field.NAME
import helper.TasDataHelper
import helper.TestBase
import navigator.CardNavigator
import navigator.screen.CardNavigatorScreen
import org.junit.jupiter.api.Assertions.assertAll
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestInfo
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
    private lateinit var cardMetadata: TasDataHelper
    private lateinit var gbaData: GbaDataHelper


    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        cardMetadata = TasDataHelper()
        gbaData = GbaDataHelper()
        CardNavigator().toScreen(CardNavigatorScreen.CardData)

        cardDataScreen = CardDataScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 The Card attributes page displays all attributes on the card. [${JIRA_ID}]")
    fun verifyCardData(testInfo: TestInfo) {
        setUp(testInfo)
        assertAll(
            { assertTrue(cardDataScreen.visible(), "card data screen is not visible") },
            { assertTrue(cardDataScreen.dataAttributeVisible(gbaData.getValueByField(FIRST_NAME, DEFAULT_BSN)), "data attribute are not visible") },
            { assertTrue(cardDataScreen.dataLabelVisible(cardMetadata.getPidClaimLabel("given_name")), "data label are not visible") },
            { assertTrue(cardDataScreen.dataAttributeVisible(gbaData.getValueByField(NAME, DEFAULT_BSN)), "data attribute are not visible") },
            { assertTrue(cardDataScreen.dataLabelVisible(cardMetadata.getPidClaimLabel("family_name")), "data label are not visible") },
            { assertTrue(cardDataScreen.dataLabelVisible(cardMetadata.getPidClaimLabel("birthdate")), "data label are not visible") },
        )
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.2 The User can go back to the Card detail page. [${JIRA_ID}]")
    fun verifyBackButton(testInfo: TestInfo) {
        setUp(testInfo)
        cardDataScreen.clickBottomBackButton()

        val cardDetailScreen = CardDetailScreen()
        assertAll(
            { assertTrue(cardDetailScreen.visible(), "card detail screen is not visible") },
        )
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @Tags(Tag("english"))
    @DisplayName("$USE_CASE.4 The Card attribute labels are multi-lingual. 5 The Card attribute values are multi-lingual if applicable and are rendered according to their schema. [${JIRA_ID}]")
    fun verifyDataLabelMultiLingual(testInfo: TestInfo) {
        setUp(testInfo)
        assertAll(
            { assertTrue(cardDataScreen.visible(), "card data screen is not visible") },
            { assertTrue(cardDataScreen.dataLabelVisible(cardMetadata.getPidClaimLabel("given_name")), "english data labels are not visible") },
            { assertTrue(cardDataScreen.dataLabelVisible(cardMetadata.getPidClaimLabel("family_name")), "english data labels are not visible") },
            { assertTrue(cardDataScreen.dataLabelVisible(cardMetadata.getPidClaimLabel("birthdate")), "english data labels are not visible") },
        )
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.6 The App provides a button for help if the data seems incorrect. This leads to a help screen explaining what to do in case of incorrect data. [${JIRA_ID}]")
    fun verifyDataIncorrectButton(testInfo: TestInfo) {
        setUp(testInfo)
        cardDataScreen.scrollToEnd()
        cardDataScreen.clickDataIncorrectButton()

        val cardDataIncorrectScreen = CardDataIncorrectScreen()
        assertTrue(cardDataIncorrectScreen.visible(), "card data incorrect screen is not visible")
    }
}
