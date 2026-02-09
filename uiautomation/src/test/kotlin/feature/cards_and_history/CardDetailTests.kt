package feature.cards_and_history

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
import screen.dashboard.DashboardScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("UC7.2 Shows card details")
class CardDetailTests : TestBase() {

    private lateinit var cardDetailScreen: CardDetailScreen
    private lateinit var cardDataScreen: CardDataScreen
    private lateinit var cardMetadata: TasDataHelper
    private lateinit var gbaData: GbaDataHelper
    private lateinit var cardDataIncorrectScreen: CardDataIncorrectScreen
    private lateinit var dashboardScreen: DashboardScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        CardNavigator().toScreen(CardNavigatorScreen.CardDetail)

        cardDetailScreen = CardDetailScreen()
        cardDataScreen = CardDataScreen()
        cardDataIncorrectScreen = CardDataIncorrectScreen()
        dashboardScreen = DashboardScreen()
        gbaData = GbaDataHelper()
        cardMetadata = TasDataHelper()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC25 Show Card Details")
    @Tags(Tag("a11yBatch1"))
    fun verifyCardDetailScreen(testInfo: TestInfo) {
        setUp(testInfo)
        assertAll(
            { assertTrue(cardDetailScreen.visible(), "card detail screen is not visible") },
            { assertTrue(cardDetailScreen.pidCardVisible(), "card face for detail screen is not visible and/or correct") },
            { assertTrue(cardDetailScreen.issuerAndHistoryStates(), "issuer and/or history state not not visible and/or correct") }
        )

        cardDetailScreen.clickCardDataButton()
        val nationalities = gbaData.getNationalities(DEFAULT_BSN)
        assertAll(
            { assertTrue(cardDataScreen.dataAttributeVisible(gbaData.getValueByField(FIRST_NAME, DEFAULT_BSN)), "data attribute are not visible") },
            { assertTrue(cardDataScreen.dataLabelVisible(cardMetadata.getPidClaimLabel("given_name")), "data label are not visible") },
            { assertTrue(cardDataScreen.dataAttributeVisible(gbaData.getValueByField(NAME, DEFAULT_BSN)), "data attribute are not visible") },
            { assertTrue(cardDataScreen.dataLabelVisible(cardMetadata.getPidClaimLabel("family_name")), "data label are not visible") },
            { assertTrue(cardDataScreen.dataLabelVisible(cardMetadata.getPidClaimLabel("birthdate")), "data label are not visible") },
            { assertTrue(cardDataScreen.dataAttributeVisible(nationalities[0]), "array attribute is not visible") },
            { assertTrue(cardDataScreen.dataAttributeVisible(nationalities[1]), "array attribute is not visible") },
            { assertTrue(cardDataScreen.dataLabelAbsent(cardMetadata.getPidClaimLabel("recovery_code")), "recovery code is visible") },
            { assertTrue(cardDataScreen.visible(), "card data screen is not visible") },
        )

        cardDataScreen.clickDataIncorrectButton()
        assertTrue(cardDataIncorrectScreen.visible(), "card data incorrect screen is not visible")

        cardDataIncorrectScreen.goBack()
        cardDataScreen.clickBottomBackButton()
        assertTrue(cardDetailScreen.visible(), "card detail screen is not visible")

        cardDetailScreen.clickBottomBackButton()
        assertTrue(dashboardScreen.visible(), "dashboard screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @Tags(Tag("english"))
    @DisplayName("LTC25 The Card attribute labels are multi-lingual")
    fun verifyDataLabelMultiLingual(testInfo: TestInfo) {
        setUp(testInfo)
        cardDetailScreen.clickCardDataButton()
        assertAll(
            { assertTrue(cardDataScreen.visible(), "card data screen is not visible") },
            { assertTrue(cardDataScreen.dataLabelVisible(cardMetadata.getPidClaimLabel("given_name")), "english data labels are not visible") },
            { assertTrue(cardDataScreen.dataLabelVisible(cardMetadata.getPidClaimLabel("family_name")), "english data labels are not visible") },
            { assertTrue(cardDataScreen.dataLabelVisible(cardMetadata.getPidClaimLabel("birthdate")), "english data labels are not visible") },
        )
    }
}
