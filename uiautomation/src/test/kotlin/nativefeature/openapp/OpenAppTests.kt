package nativefeature.openapp

import helper.TestBase
import nativescreen.introduction.IntroductionScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("UC1.2 Open the App")
class OpenAppTests : TestBase() {

    private lateinit var introductionScreen: IntroductionScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        introductionScreen = IntroductionScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @Tags(Tag("dutch"))
    @DisplayName("LTC42 If the device language is set to Dutch, then the app starts in Dutch.")
    fun verifyDutchLanguage(testInfo: TestInfo) {
        setUp(testInfo)
        assertTrue(introductionScreen.nextButtonTextVisible("Volgende"))
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @Tags(Tag("english"))
    @DisplayName("LTC42 If the device language is set to English, then the app starts in English.")
    fun verifyEnglishLanguage(testInfo: TestInfo) {
        setUp(testInfo)
        assertTrue(introductionScreen.nextButtonTextVisible("Next"))
    }

    /**
     * 3. When a PIN has been set up before, the App starts UC 2.3 Unlock the App.
     */

    /**
     * 4. When a PID has been issued, the App starts show dashbaord.
     */
}
