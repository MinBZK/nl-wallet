package screen.web.digid

import org.openqa.selenium.By
import org.openqa.selenium.JavascriptExecutor
import util.MobileActions

class DigidLoginStartWebPage : MobileActions() {

    private val headlineTextLocator = By.xpath("//*[contains(text(), 'Hoe wilt u inloggen?')]")
    private val mockLoginButtonLocator = By.linkText("Inloggen met DigiD mock")

    fun visible(): Boolean {
        return isWebElementVisible(findWebElement(headlineTextLocator))
    }

    fun clickMockLoginButton() {
        switchToWebViewContext()
        val element = findWebElement(mockLoginButtonLocator)
        (driver as JavascriptExecutor).executeScript("arguments[0].click()", element)
    }
}
