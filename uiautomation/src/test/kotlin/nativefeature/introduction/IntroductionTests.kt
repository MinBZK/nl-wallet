package nativefeature.introduction

import helper.TestBase
import nativescreen.introduction.IntroductionPrivacyScreen
import nativescreen.introduction.IntroductionScreen
import nativescreen.introduction.PrivacyPolicyScreen
import nativescreen.security.PinScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("UC 1.1 Introduce the app")
class IntroductionTests : TestBase() {

    private lateinit var introductionScreen: IntroductionScreen
    private lateinit var privacyScreen: IntroductionPrivacyScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        introductionScreen = IntroductionScreen()
        privacyScreen = IntroductionPrivacyScreen()

    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("UC 1.1 LTC13 Introduction Happy flow")
    fun verifyWelcomeScreen(testInfo: TestInfo) {
        setUp(testInfo)
        assertTrue(introductionScreen.page1Visible(), "page 1 is not visible")

        introductionScreen.clickNextButton() // page 1 -> 2
        assertTrue(introductionScreen.page2Visible(), "page 2 is not visible")

        introductionScreen.clickNextButton() // page 2 -> 3
        assertTrue(introductionScreen.page3Visible(), "page 3 is not visible")

        introductionScreen.clickNextButton() // page 3 -> privacy
        assertTrue(privacyScreen.visible(), "privacy screen is not visible")

        privacyScreen.clickPrivacyButton()

        val privacyPolicyScreen = PrivacyPolicyScreen()
        assertTrue(privacyPolicyScreen.visible(), "privacy policy screen is not visible")

        privacyPolicyScreen.clickBackButton()

        privacyScreen.clickNextButton()

        val pinScreen = PinScreen()
        assertTrue(pinScreen.setupPinScreenVisible(), "choose pin screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("UC 1.1 LTC14 User skips introduction")
    fun verifySkipIntroButton(testInfo: TestInfo) {
        setUp(testInfo)

        // Skip from page 1
        introductionScreen.clickSkipButton() // page 1 -> privacy
        assertTrue(privacyScreen.visible(), "privacy screen is not visible")

        // Back to page 1, next, skip from page 2
        privacyScreen.clickBackButton() // privacy -> page 1
        assertTrue(introductionScreen.page1Visible(), "page 1 is not visible")
        introductionScreen.clickNextButton() // page 1 -> page 2
        assertTrue(introductionScreen.page2Visible(), "page 2 is not visible")
        introductionScreen.clickSkipButton() // page 2 -> privacy
        assertTrue(privacyScreen.visible(), "privacy screen is not visible")

        // Back to page 2, next to page 3
        privacyScreen.clickBackButton() // privacy -> page 2
        assertTrue(introductionScreen.page2Visible(), "page 2 is not visible")
        introductionScreen.clickNextButton() // page 2 -> page 3
        assertTrue(introductionScreen.page3Visible(), "page 3 is not visible")

        // Page 3 only contains next button (skip button is gone on last intro page)
        introductionScreen.clickNextButton() // page 3 -> privacy
        assertTrue(privacyScreen.visible(), "privacy screen is not visible")

        introductionScreen.clickBackButton() // page 3 -> 2
        assertTrue(introductionScreen.page3Visible(), "page 3 is not visible")

        introductionScreen.clickBackButton() // page 3 -> 2
        assertTrue(introductionScreen.page2Visible(), "page 2 is not visible")

        introductionScreen.clickBackButton() // page 2 -> 1
        assertTrue(introductionScreen.page1Visible(), "page 1 is not visible")
    }
}
