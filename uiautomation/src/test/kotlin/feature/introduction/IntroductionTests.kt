package feature.introduction

import helper.TestBase
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Nested
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.introduction.IntroductionPrivacyScreen
import screen.introduction.IntroductionScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${IntroductionTests.USE_CASE} App displays introductory information [${IntroductionTests.JIRA_ID}]")
class IntroductionTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 1.1"
        const val JIRA_ID = "PVW-1218"
    }

    private lateinit var introductionScreen: IntroductionScreen

    fun setUp() {
        introductionScreen = IntroductionScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1. The App shows a welcome screen so the user knows they are using the NL wallet. [${JIRA_ID}]")
    fun verifyWelcomeScreen() {
        setUp()
        assertTrue(introductionScreen.page1Visible(), "page 1 is not visible")
    }

    @Nested
    @DisplayName("$USE_CASE.2 The App shows a series of explanation screens containing the following: [${JIRA_ID}]")
    inner class IntroductoryScreens {

        @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
        @DisplayName("$USE_CASE.2.1 The security benefits of the app (online identification). [${JIRA_ID}]")
        fun verifySecurityScreen() {
            setUp()
            introductionScreen.clickNextButton() // page 1 -> 2
            assertTrue(introductionScreen.page2Visible(), "page 2 is not visible")
        }

        @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
        @DisplayName("$USE_CASE.2.2 The privacy benefits of the app (selective disclosure). [${JIRA_ID}]")
        fun verifyPrivacyScreen() {
            setUp()
            introductionScreen.clickNextButton() // page 1 -> 2
            assertTrue(introductionScreen.page2Visible(), "page 2 is not visible")

            introductionScreen.clickNextButton() // page 2 -> 3
            assertTrue(introductionScreen.page3Visible(), "page 3 is not visible")
        }
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.3 The App offers a button to skip the intro, leading to the privacy summary. [${JIRA_ID}]")
    @Tags(Tag("smoke"))
    fun verifySkipIntroButton() {
        setUp()
        val privacyScreen = IntroductionPrivacyScreen()

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
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.4 The explanation screens all display a back-button. [${JIRA_ID}]")
    fun verifyPageBackButtons() {
        setUp()
        introductionScreen.clickNextButton() // page 1 -> 2
        assertTrue(introductionScreen.page2Visible(), "page 2 is not visible")

        introductionScreen.clickNextButton() // page 2 -> 3
        assertTrue(introductionScreen.page3Visible(), "page 3 is not visible")

        introductionScreen.clickBackButton() // page 3 -> 2
        assertTrue(introductionScreen.page2Visible(), "page 2 is not visible")

        introductionScreen.clickBackButton() // page 2 -> 1
        assertTrue(introductionScreen.page1Visible(), "page 1 is not visible")
    }
}
