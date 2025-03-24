package feature.web.digid

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junitpioneer.jupiter.RetryingTest
import screen.web.digid.DigidLoginMockWebPage
import screen.web.digid.DigidLoginStartWebPage

class MockDigidWebTests : TestBase() {

    private lateinit var digidLoginStartWebPage: DigidLoginStartWebPage

    fun setUp() {
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.DigidLoginStartWebPage)

        digidLoginStartWebPage = DigidLoginStartWebPage()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    fun verifyMockDigidLogin() {
        setUp()
        assertTrue(digidLoginStartWebPage.visible(), "digid login start web page is not visible")

        digidLoginStartWebPage.clickMockLoginButton()

        val digidLoginMockWebPage = DigidLoginMockWebPage()
        assertTrue(digidLoginMockWebPage.visible(), "digid login mock web page is not visible")
//        Since a BSN is already prefilled entering is not necessary; for iOS entering caused more effort therefore this is commented out now.
//        digidLoginMockWebPage.enterBsn("999991771")
        digidLoginMockWebPage.clickLoginButton()
    }
}
