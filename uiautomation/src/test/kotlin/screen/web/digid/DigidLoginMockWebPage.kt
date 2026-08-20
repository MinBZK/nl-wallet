package screen.web.digid

import org.openqa.selenium.By
import util.MobileActions

class DigidLoginMockWebPage : MobileActions() {

    private val headlineTextLocator = By.xpath("//*contains(@class, \"intro\")")
    private val mockLoginButtonLocator = By.xpath("//*[@id='custom-bsn-submit']")
    private val bsnInput = By.xpath("//*[@id='custom-bsn']")

    fun visible(): Boolean {
        return isWebElementVisible(findWebElement(headlineTextLocator))
    }

    fun enterBsn(bsn: String) {
        isWebElementVisible(findWebElement(bsnInput))
        findWebElement(bsnInput).clear()
        findWebElement(bsnInput).sendKeys(bsn)
    }

    fun clickLoginButton() {
        isWebElementVisible(findWebElement(mockLoginButtonLocator))
        clickWebElementWithMouseEvent(findWebElement(mockLoginButtonLocator))
    }

    fun login(bsn: String) {
        switchToWebViewContext()
        enterBsn(bsn)
        clickLoginButton()
    }
}
