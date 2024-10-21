package screen.web.rp

import org.openqa.selenium.By
import screen.web.rp.shared.RelyingPartyDemoBody
import util.MobileActions

class RelyingPartyMarketplaceWebPage : MobileActions() {

    val body = RelyingPartyDemoBody()

    private val customStartButtonLocator = By.xpath("//nl-wallet-button[@text='Verder met NL Wallet']")

    fun customStartButtonVisible() = isWebElementVisible(findElement(customStartButtonLocator))
}
