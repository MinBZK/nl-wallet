package screen.web.demo.issuer

import org.openqa.selenium.By
import util.MobileActions

class IssuerConsentWebPage : MobileActions() {

    private val giveConsentButtonLocator = By.xpath("//section[@class='buttons']//button")

    fun switchToConsentPage() = switchToWebViewWindowContaining(giveConsentButtonLocator)

    fun visible() = isWebElementVisible(findWebElement(giveConsentButtonLocator))

    fun clickGiveConsentButton() = clickWebElement(findWebElement(giveConsentButtonLocator))
}
