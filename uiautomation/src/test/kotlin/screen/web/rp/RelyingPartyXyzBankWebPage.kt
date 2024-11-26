package screen.web.rp

import org.openqa.selenium.By
import screen.web.rp.shared.RelyingPartyDemoBody
import util.MobileActions

class RelyingPartyXyzBankWebPage : MobileActions() {

    val body = RelyingPartyDemoBody()

    private val customStartButtonLocator = By.xpath("//nl-wallet-button[@text='Gebruik NL Wallet']")

    fun customStartButtonVisible() = isWebElementVisible(findElement(customStartButtonLocator))
}
