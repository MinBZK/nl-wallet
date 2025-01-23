package feature.web.rp

import helper.TestBase
import navigator.MenuNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Nested
import org.junitpioneer.jupiter.RetryingTest
import screen.disclosure.DisclosureApproveOrganizationScreen
import screen.menu.MenuScreen
import screen.web.rp.RelyingPartyAmsterdamWebPage
import screen.web.rp.RelyingPartyMarketplaceWebPage
import screen.web.rp.RelyingPartyMonkeyBikeWebPage
import screen.web.rp.RelyingPartyOverviewWebPage
import screen.web.rp.RelyingPartyXyzBankWebPage

class RelyingPartyWebTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 13.1"
        const val JIRA_ID = "PVW-2541"
    }

    private lateinit var overviewWebPage: RelyingPartyOverviewWebPage

    @BeforeEach
    fun setUp() {
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()

        overviewWebPage = RelyingPartyOverviewWebPage()
    }

    /**
     * 1. The OV includes a frontend library that can be embedded by the Verifier in their front-end application.
     * >> This requirement hard, if not impossible to be tested in an e2e setup.
     */

    /**
     * 2. The library offers functionality for managing sessions on the OV-API. Initially: 1) Starting the session, and  2) Subscribing to the final success/failure session state event.
     * >> This requirement hard, if not impossible to be tested in an e2e setup.
     */

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.3 The library offers a standardized Start-button, the Verifier decides which button text to display. [$JIRA_ID]")
    fun verifyCustomStartButtonTexts() {
        val amsterdamWebPage = RelyingPartyAmsterdamWebPage()
        val xyzBankWebPage = RelyingPartyXyzBankWebPage()
        val marketplaceWebPage = RelyingPartyMarketplaceWebPage()
        val monkeyBikeWebPage = RelyingPartyMonkeyBikeWebPage()

        // Amsterdam
        overviewWebPage.clickAmsterdamButton()
        assertTrue(amsterdamWebPage.customStartButtonVisible(), "Custom start button is not visible")
        amsterdamWebPage.body.clickBackButton()

        // XYZ Bank
        overviewWebPage.clickXyzBankButton()
        assertTrue(xyzBankWebPage.customStartButtonVisible(), "Custom start button is not visible")
        xyzBankWebPage.body.clickBackButton()
        // Marketplace
        overviewWebPage.clickMarketplaceButton()
        assertTrue(marketplaceWebPage.customStartButtonVisible(), "Custom start button is not visible")
        marketplaceWebPage.body.clickBackButton()

        // MonkeyBike
        overviewWebPage.clickMonkeyBikeButton()
        assertTrue(monkeyBikeWebPage.customStartButtonVisible(), "Custom start button is not visible")
        monkeyBikeWebPage.body.clickBackButton()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.4 When a user clicks one of these buttons, the library requests a new disclosure session from the Relying Party backend (to be implemented by the RP). The RP backend should request a new session from the OV (PVW-2464) and return the information to the library. [$JIRA_ID]")
    fun verifyStartButtonClick() {
        val amsterdamWebPage = RelyingPartyAmsterdamWebPage()

        // Amsterdam
        overviewWebPage.clickAmsterdamButton()
        amsterdamWebPage.body.clickStartButton()
        assertTrue(amsterdamWebPage.popup.deviceChoiceDutchTextVisible(), "Device choice popup is not visible")
    }

    /**
     * 5. When the frontend library tries to fetch the session status, but this takes too long or fails, the user is warned that they  may not have a good internet connection and offers to try again.
     * >> This requirement hard, if not impossible to be tested in an e2e setup.
     */

    @Nested
    @DisplayName("$USE_CASE.6 When a new session was started, the library attempts to distinguish between same-device and cross-device flow, as follows: [$JIRA_ID]")
    inner class NewSession {

        @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
        @DisplayName(
            "$USE_CASE.6.1  When a mobile device is detected, and when the library cannot reliably detect that it runs on a desktop device, it asks the user where the NL Wallet is installed, offering the following options:\n" +
                " 1. The option 'on this device' triggers the same-device flow.\n" +
                " 2. The option 'on another device' triggers the cross-device flow.\n" +
                " 3. The option abort aborts the session."
        )
        fun verifySameOrCrossDeviceFlow() {
            val amsterdamWebPage = RelyingPartyAmsterdamWebPage()

            // Amsterdam
            overviewWebPage.clickAmsterdamButton()
            amsterdamWebPage.body.clickStartButton()
            assertTrue(amsterdamWebPage.popup.deviceChoiceDutchTextVisible(), "Device choice options are not visible")
            assertTrue(amsterdamWebPage.popup.sameDeviceButtonVisible(), "Same device button is not visible")
            assertTrue(amsterdamWebPage.popup.otherDeviceButtonVisible(), "Other device button is not visible")
            assertTrue(amsterdamWebPage.popup.closeButtonVisible(), "Close button is not visible")
        }

        /**
         * 6.2. When the library can reliably detect that it runs on a desktop device, it automatically starts the cross-device flow.
         * >> Manual test: https://SSSS/browse/PVW-3592
         */
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.7 In the same-device flow, the library forwards the user to the Universal Link which either opens the installed NL Wallet app. [$JIRA_ID]")
    fun verifyUniversalLink() {
        val amsterdamWebPage = RelyingPartyAmsterdamWebPage()

        // Amsterdam
        overviewWebPage.clickAmsterdamButton()
        amsterdamWebPage.body.clickStartButton()
        amsterdamWebPage.popup.clickSameDeviceButton()

        // App
        val disclosureScreen = DisclosureApproveOrganizationScreen()
        assertTrue(disclosureScreen.loginAmsterdamTitleVisible(), "Login Amsterdam flow is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.8 In the cross-device flow, the library displays the QR code received from the OV and instructs the user to scan the QR code using the wallet. [$JIRA_ID]")
    fun verifyQrVisible() {
        val amsterdamWebPage = RelyingPartyAmsterdamWebPage()

        // Amsterdam
        overviewWebPage.clickAmsterdamButton()
        amsterdamWebPage.body.clickStartButton()
        amsterdamWebPage.popup.clickOtherDeviceButton()
        assertTrue(amsterdamWebPage.popup.scanQrTextVisible(), "Scan QR text is not visible")
        assertTrue(amsterdamWebPage.popup.qrVisible(), "QR code is not visible")
    }

    /**
     * 8.1. The QR is automatically refreshed every 2 second to prevent a passive attacker from just relaying the QR code to (potential) victims.
     * >> Manual test: https://SSSS/browse/PVW-3594
     */

    /**
     * 9. The library polls the status of this session (PVW-2465) and upon status change it does the following...
     * >> Manual test: https://SSSS/browse/PVW-3593
     */

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.10  The library continuously displays a link to an online help page that explains what the NL Wallet is and how to use it. For now, the text on this page is limited. [$JIRA_ID]")
    fun verifyHelpSectionVisible() {
        val amsterdamWebPage = RelyingPartyAmsterdamWebPage()

        // Amsterdam
        overviewWebPage.clickAmsterdamButton()
        amsterdamWebPage.body.clickStartButton()
        assertTrue(amsterdamWebPage.popup.helpSectionVisible(), "Help section is not visible")

        amsterdamWebPage.popup.clickOtherDeviceButton()
        assertTrue(amsterdamWebPage.popup.helpSectionVisible(), "Help section is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.11 The library supports the following languages: Dutch, English. The language to be used is specified by the relying party. [$JIRA_ID]")
    fun verifyMultilanguage() {
        val amsterdamWebPage = RelyingPartyAmsterdamWebPage()

        // Amsterdam
        overviewWebPage.clickAmsterdamButton()

        // Amsterdam; English
        amsterdamWebPage.header.clickEnglishLanguageButton()
        assertTrue(amsterdamWebPage.englishTextsVisible(), "English texts are not visible")

        amsterdamWebPage.body.clickStartButton()
        assertTrue(amsterdamWebPage.popup.deviceChoiceEnglishTextVisible(), "English text are not visible")
        amsterdamWebPage.popup.clickCloseButton()

        // Amsterdam; Dutch
        amsterdamWebPage.header.clickDutchLanguageButton()
        assertTrue(amsterdamWebPage.dutchTextsVisible(), "Dutch texts are not visible")

        amsterdamWebPage.body.clickStartButton()
        assertTrue(amsterdamWebPage.popup.deviceChoiceDutchTextVisible(), "Dutch text are not visible")
    }
}
