package screen.web.demo.rp

import org.openqa.selenium.By
import screen.web.demo.shared.DemoBody
import screen.web.demo.shared.DemoPopup
import util.MobileActions

class RelyingPartyAmsterdamWebPage : MobileActions() {

    val body = DemoBody()
    val popup = DemoPopup()

    private val loggedInHeaderTextDutchLocator = By.xpath("//h2[text()='Welkom in Mijn Amsterdam']")

    fun loggedInMessageVisible() = isWebElementVisible(findWebElement(loggedInHeaderTextDutchLocator))

    fun openSameDeviceWalletFlow() {
        body.clickStartButton()
        popup.clickSameDeviceButton()
    }
}
