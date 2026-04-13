package feature.cards_and_history

import helper.TasDataHelper
import helper.TestBase
import navigator.MenuNavigator
import navigator.screen.MenuNavigatorScreen
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
import screen.card.CardDeletedScreen
import screen.card.CardDetailScreen
import screen.dashboard.DashboardScreen
import screen.history.HistoryOverviewScreen
import screen.issuance.CardIssuanceScreen
import screen.issuance.DisclosureIssuanceScreen
import screen.menu.MenuScreen
import screen.security.PinScreen
import screen.web.demo.DemoIndexWebPage
import screen.web.demo.issuer.IssuerWebPage

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("UC7.4 Delete card")
class DeleteCardTests : TestBase() {

    private lateinit var cardDetailScreen: CardDetailScreen
    private lateinit var cardDataScreen: CardDataScreen
    private lateinit var cardMetadata: TasDataHelper
    private lateinit var cardDataIncorrectScreen: CardDataIncorrectScreen
    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var indexWebPage: DemoIndexWebPage
    private lateinit var issuerWebPage: IssuerWebPage
    private lateinit var disclosureForIssuanceScreen: DisclosureIssuanceScreen
    private lateinit var cardIssuanceScreen: CardIssuanceScreen
    private lateinit var pinScreen: PinScreen
    private lateinit var cardDeletedScreen: CardDeletedScreen
    private lateinit var menuScreen: MenuScreen
    private lateinit var historyOverviewScreen: HistoryOverviewScreen


    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)

        cardDetailScreen = CardDetailScreen()
        cardDataScreen = CardDataScreen()
        cardDataIncorrectScreen = CardDataIncorrectScreen()
        dashboardScreen = DashboardScreen()
        indexWebPage = DemoIndexWebPage()
        issuerWebPage = IssuerWebPage()
        cardMetadata = TasDataHelper()
        disclosureForIssuanceScreen = DisclosureIssuanceScreen()
        cardIssuanceScreen = CardIssuanceScreen()
        pinScreen = PinScreen()
        cardDeletedScreen = CardDeletedScreen()
        menuScreen = MenuScreen()
        historyOverviewScreen = HistoryOverviewScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC79 Delete card")
    @Tags(Tag("a11yBatch1"))
    fun verifyCardDetailScreen(testInfo: TestInfo) {
        setUp(testInfo)

        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickInsuranceButton()
        issuerWebPage.openSameDeviceWalletFlow()

        disclosureForIssuanceScreen.switchToNativeContext()
        disclosureForIssuanceScreen.share()
        pinScreen.enterPin(DEFAULT_PIN)
        cardIssuanceScreen.clickAddCardButton()
        pinScreen.enterPin(DEFAULT_PIN)
        cardIssuanceScreen.clickToDashboardButton();
        dashboardScreen.scrollToEndOfScreen()

        dashboardScreen.clickCard(cardMetadata.getInsuranceDisplayName())
        cardDetailScreen.clickDeleteCardButton()
        cardDetailScreen.clickConfirmDeleteCard()
        pinScreen.enterPin(DEFAULT_PIN)
        assertTrue(cardDeletedScreen.visible(), "Card deleted screen is not visible")

        cardDeletedScreen.clickToDashboardButton()
        dashboardScreen.scrollToEndOfScreen()
        assertTrue(!dashboardScreen.cardVisible(cardMetadata.getInsuranceDisplayName()), "Deleted card is still visible")

        dashboardScreen.clickMenuButton()
        menuScreen.clickHistoryButton()
        assertTrue(historyOverviewScreen.cardDeletedEventVisible(), "history event is not visible")

    }
}
