package screen.web.digid

import org.openqa.selenium.By
import util.MobileActions

class DigidLoginStartWebPage : MobileActions() {

    private val headlineTextLocator = By.xpath("//*[contains(text(), 'Hoe wilt u inloggen?')]")
    private val mockLoginButtonLocator = By.linkText("Inloggen met DigiD mock")

    fun visible(): Boolean {
        return isWebElementVisible(findWebElement(headlineTextLocator))
    }

    fun clickMockLoginButton() {
        switchToWebViewContext()
        clickWebElement(findWebElement(mockLoginButtonLocator))
    }
}
