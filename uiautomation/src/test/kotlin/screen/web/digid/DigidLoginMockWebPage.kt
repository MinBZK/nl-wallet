package screen.web.digid

import org.openqa.selenium.By
import util.MobileActions

class DigidLoginMockWebPage : MobileActions() {

    private val headlineTextLocator = By.xpath("//*[text()='DigiD MOCK']")
    private val bsnInputLocator = By.id("bsn_inp")
    private val mockLoginButtonLocator = By.linkText("Login / Submit")

    fun visible() = isWebElementVisible(findElement(headlineTextLocator))

    fun enterBsn(bsn: String) {
        val bsnInput = findElement(bsnInputLocator)
        bsnInput.clear()
        bsnInput.sendKeys(bsn)
    }

    fun clickLoginButton() {
        findElement(mockLoginButtonLocator).click()
        switchToAppContext()
    }
}
