package feature.digid

import helper.TestBase
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junitpioneer.jupiter.RetryingTest
import screen.digid.DigidLoginMockWebPage
import screen.digid.DigidLoginStartWebPage
import setup.OnboardingNavigator
import setup.Screen

class MockDigidWebTests : TestBase() {

    private val onboardingNavigator = OnboardingNavigator()

    private lateinit var digidLoginStartWebPage: DigidLoginStartWebPage

    @BeforeEach
    fun setUp() {
        onboardingNavigator.toScreen(Screen.DigidLoginStartWebPage)

        digidLoginStartWebPage = DigidLoginStartWebPage()
    }

    @RetryingTest(MAX_RETRY_COUNT)
    fun verifyMockDigidLogin() {
        assertTrue(digidLoginStartWebPage.visible(), "digid login start web page is not visible")

        digidLoginStartWebPage.clickMockLoginButton()

        val digidLoginMockWebPage = DigidLoginMockWebPage()
        assertTrue(digidLoginMockWebPage.visible(), "digid login mock web page is not visible")

        digidLoginMockWebPage.enterBsn("999991771")
        digidLoginMockWebPage.clickLoginButton()
    }
}
