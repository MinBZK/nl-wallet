package screen.web.rp

import org.openqa.selenium.By
import screen.web.rp.shared.RelyingPartyDemoBody
import screen.web.rp.shared.RelyingPartyDemoHeader
import screen.web.rp.shared.RelyingPartyDemoPopup
import util.MobileActions

class RelyingPartyAmsterdamWebPage : MobileActions() {

    val header = RelyingPartyDemoHeader()
    val body = RelyingPartyDemoBody()
    val popup = RelyingPartyDemoPopup()

    private val headerTextDutchLocator = By.xpath("//h2[text()='Inloggen op Mijn Amsterdam']")
    private val customStartButtonDutchLocator = By.xpath("//nl-wallet-button[@text='Inloggen met NL Wallet']")

    private val headerTextEnglishLocator = By.xpath("//h2[text()='Login to Mijn Amsterdam']")
    private val customStartButtonEnglishLocator = By.xpath("//nl-wallet-button[@text='Login with NL Wallet']")

    fun dutchTextsVisible() =
        isWebElementVisible(findElement(headerTextDutchLocator)) &&
            isWebElementVisible(findElement(customStartButtonDutchLocator))

    fun englishTextsVisible() =
        isWebElementVisible(findElement(headerTextEnglishLocator)) &&
            isWebElementVisible(findElement(customStartButtonEnglishLocator))

    fun customStartButtonVisible() = isWebElementVisible(findElement(customStartButtonDutchLocator))
}
