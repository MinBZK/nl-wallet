package screen.digid

import org.openqa.selenium.By
import util.MobileActions

class DigidLoginStartWebPage : MobileActions() {

    private val headlineTextLocator = By.xpath("//*[contains(text(), 'Inloggen NL Wallet')]")

    private val mockLoginButtonLocator = By.linkText("Inloggen met DigiD MOCK")

    fun visible() = isWebElementVisible(findElement(headlineTextLocator))

    fun clickMockLoginButton() = findElement(mockLoginButtonLocator).click()
}
