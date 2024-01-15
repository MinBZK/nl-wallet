package feature.menu

import helper.TestBase
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Nested
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.Test
import screen.card.CardOverviewScreen
import screen.change_language.ChangeLanguageScreen
import screen.digid.DigidLoginMockWebPage
import screen.digid.DigidLoginStartWebPage
import screen.introduction.IntroductionConditionsScreen
import screen.introduction.IntroductionExpectationsScreen
import screen.introduction.IntroductionPrivacyScreen
import screen.introduction.IntroductionScreen
import screen.menu.MenuScreen
import screen.personalize.PersonalizeInformScreen
import screen.personalize.PersonalizePidPreviewScreen
import screen.personalize.PersonalizeSuccessScreen
import screen.security.PinScreen
import screen.security.SetupSecurityCompletedScreen
import screen.settings.SettingsScreen

@DisplayName("UC 9.3 - User changes language [PVW-1224]")
class ChangeLanguageTests : TestBase() {

    private val chosenPin = "122222"

    private lateinit var changeLanguageScreen: ChangeLanguageScreen

    @BeforeEach
    fun setUp() {
        val introductionScreen = IntroductionScreen()
        val expectationsScreen = IntroductionExpectationsScreen()
        val privacyScreen = IntroductionPrivacyScreen()
        val conditionsScreen = IntroductionConditionsScreen()
        val pinScreen = PinScreen()
        val setupSecurityCompletedScreen = SetupSecurityCompletedScreen()
        val personalizeInformScreen = PersonalizeInformScreen()
        val digidLoginStartWebPage = DigidLoginStartWebPage()
        val digidLoginMockWebPage = DigidLoginMockWebPage()
        val personalizePidPreviewScreen = PersonalizePidPreviewScreen()
        val personalizeSuccessScreen = PersonalizeSuccessScreen()
        val cardOverviewScreen = CardOverviewScreen()
        val menuScreen = MenuScreen()
        val settingsScreen = SettingsScreen()

        // Start all tests on digid login start web page
        introductionScreen.clickSkipButton()
        expectationsScreen.clickNextButton()
        privacyScreen.clickNextButton()
        conditionsScreen.clickNextButton()
        pinScreen.enterPin(chosenPin)
        pinScreen.enterPin(chosenPin)
        setupSecurityCompletedScreen.clickNextButton()
        personalizeInformScreen.clickLoginWithDigidButton()
        personalizeInformScreen.switchToWebView()
        digidLoginStartWebPage.clickMockLoginButton()
        digidLoginMockWebPage.clickLoginButton()
        personalizePidPreviewScreen.switchToApp()
        personalizePidPreviewScreen.clickAcceptButton()
        pinScreen.enterPin(chosenPin)
        personalizeSuccessScreen.clickNextButton()
        cardOverviewScreen.clickMenuButton()
        menuScreen.clickSettingsButton()
        settingsScreen.clickChangeLanguageButton()

        changeLanguageScreen = ChangeLanguageScreen()
    }

    @Test
    @DisplayName("1. App settings menu displays option to change language.")
    fun verifyChangeLanguageScreen() {
        assertTrue(changeLanguageScreen.visible(), "change language screen is not visible")
    }

    @Test
    @DisplayName("2. Language screen offers two options: English & Dutch.")
    fun verifyLanguageButtonsVisible() {
        assertTrue(changeLanguageScreen.languageButtonsVisible(), "language buttons are not visible")
    }

    @Nested
    @DisplayName("3. When the User selects a language, the app immediately uses the newly selected language.")
    inner class LanguageChange {

        @Test
        @Tags(Tag("english"))
        @DisplayName("3.1. When the User selects Dutch, the app immediately uses Dutch.")
        fun verifyDutchLanguageSelect() {
            assertTrue(changeLanguageScreen.englishScreenTitleVisible(), "english screen title is not visible")
            changeLanguageScreen.clickDutchButton()

            assertTrue(changeLanguageScreen.dutchScreenTitleVisible(), "dutch screen title is not visible")
        }

        @Test
        @Tags(Tag("dutch"))
        @DisplayName("3.2. When the User selects English, the app immediately uses English.")
        fun verifyEnglishLanguageSelect() {
            assertTrue(changeLanguageScreen.dutchScreenTitleVisible(), "dutch screen title is not visible")
            changeLanguageScreen.clickEnglishButton()

            assertTrue(changeLanguageScreen.englishScreenTitleVisible(), "english screen title is not visible")
        }
    }
}
