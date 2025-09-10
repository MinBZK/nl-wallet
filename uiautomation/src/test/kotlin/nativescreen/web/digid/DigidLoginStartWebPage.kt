package nativescreen.web.digid

import org.openqa.selenium.By
import util.NativeMobileActions

class DigidLoginStartWebPage : NativeMobileActions() {

    private val headlineTextLocator = By.xpath("//*[contains(text(), 'Hoe wilt u inloggen?')]")
    private val mockLoginButtonLocator = By.linkText("Inloggen met DigiD mock")

    fun visible(): Boolean {
        return isWebElementVisible(findElement(headlineTextLocator))
    }

    fun clickMockLoginButton() {
        clickWebElement(findElement(mockLoginButtonLocator))
    }
}
