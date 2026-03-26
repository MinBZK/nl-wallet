package screen.web.revocation_portal

import org.openqa.selenium.By
import util.MobileActions

class RevocationPortalWebPage : MobileActions() {

    private val headerTextLocator = By.xpath("//h1")
    private val submitButton = By.xpath("//button[@class='btn btn-delete']")
    private val revocationCodeField = By.xpath("//input[@id='deletion-code']")
    private val succesMessage = By.xpath("//p[@id='success_message']")

    fun visible() = isWebElementVisible(findWebElement(headerTextLocator))

    fun revokeWallet(revocationCode: String) {
        findWebElement(revocationCodeField).sendKeys(revocationCode)
        clickWebElement(findWebElement(submitButton))
    }

    fun successMessageVisible(): Boolean {
        val timestamp = findWebElement(succesMessage).getAttribute("data-revoked-at")
        return isWebElementVisible(findWebElement(succesMessage)) && timestamp?.isNotEmpty() == true
    }

}
