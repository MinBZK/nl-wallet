package screen.web.demo.rp

import org.openqa.selenium.By
import screen.web.demo.shared.DemoBody
import screen.web.demo.shared.DemoPopup
import util.MobileActions

class RelyingPartyMarketplaceWebPage : MobileActions() {

    val body = DemoBody()
    val popup = DemoPopup()

    private val accountWelcomeTextDutchLocator = By.xpath("//div[@role='alert' and contains(./span/text(), 'Welkom')]")

    fun welcomeMessageVisible() = isWebElementVisible(findWebElement(accountWelcomeTextDutchLocator))

    fun openSameDeviceWalletFlow() {
        body.clickStartButton()
        popup.clickSameDeviceButton()
    }
}
