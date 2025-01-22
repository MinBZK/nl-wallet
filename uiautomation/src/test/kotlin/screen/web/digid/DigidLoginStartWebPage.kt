package screen.web.digid

import org.openqa.selenium.By
import util.MobileActions

class DigidLoginStartWebPage : MobileActions() {

    private val headlineTextLocator = By.xpath("//*[contains(text(), 'Hoe wilt u inloggen?')]")
    private val mockLoginButtonLocator = By.linkText("Inloggen met DigiD mock")

    fun visible(): Boolean {
        Thread.sleep(PAGE_LOAD_TIMEOUT)
        return isWebElementVisible(findElement(headlineTextLocator))
    }

    fun clickMockLoginButton() {
        clickWebElement(findElement(mockLoginButtonLocator))
    }
}
