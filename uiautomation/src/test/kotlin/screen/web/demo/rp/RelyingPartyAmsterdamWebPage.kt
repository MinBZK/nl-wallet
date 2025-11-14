package screen.web.demo.rp

import org.openqa.selenium.By
import screen.web.demo.shared.DemoBody
import screen.web.demo.shared.DemoPopup
import util.MobileActions

class RelyingPartyAmsterdamWebPage : MobileActions() {

    val body = DemoBody()
    val popup = DemoPopup()

    private val loggedInHeaderTextLocator = By.xpath("//div[contains(@class, 'notification ')]")

    fun loggedInMessageVisible() = isWebElementVisible(findWebElement(loggedInHeaderTextLocator))

    fun openSameDeviceWalletFlow() {
        body.clickStartButton()
        popup.clickSameDeviceButton()
    }
}
