package feature.openapp

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.demo.DemoScreen
import screen.introduction.IntroductionScreen
import screen.issuance.PersonalizeInformScreen
import screen.security.PinScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("UC1.2 Open the App")
class OpenAppTests : TestBase() {

    private lateinit var introductionScreen: IntroductionScreen
    private lateinit var pinScreen: PinScreen
    private lateinit var personalizeInformScreen: PersonalizeInformScreen
    private lateinit var demoScreen: DemoScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        introductionScreen = IntroductionScreen()
        pinScreen = PinScreen()
        personalizeInformScreen = PersonalizeInformScreen()
        demoScreen = DemoScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC32 If the device language is set to Dutch, then the app starts in Dutch.")
    fun verifyDutchLanguage(testInfo: TestInfo) {
        setUp(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.Introduction)
        assertTrue(introductionScreen.nextButtonTextVisible("Volgende"))
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @Tags(Tag("english"))
    @DisplayName("LTC32 If the device language is set to English, then the app starts in English.")
    fun verifyEnglishLanguage(testInfo: TestInfo) {
        setUp(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.Introduction)
        assertTrue(introductionScreen.nextButtonTextVisible("Next"))
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC44 Wallet not created when app is opened, PIN has not been setup")
    fun verifyOpenAppWithoutPinSetup(testInfo: TestInfo) {
        setUp(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.SecurityChoosePin)
        pinScreen.closeApp()
        pinScreen.openApp()
        assertTrue(demoScreen.visible(), "Demo screen not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC44 Wallet not created when app is opened, PID has not been issued")
    fun verifyOpenAppWithoutPID(testInfo: TestInfo) {
        setUp(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.PersonalizeConfirmIssuance)
        personalizeInformScreen.closeApp()
        personalizeInformScreen.openApp()
        assertTrue(pinScreen.pinScreenVisible(), "Pin screen not visible")
    }
}
