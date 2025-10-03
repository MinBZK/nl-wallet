package screen.web.demo.rp

import org.openqa.selenium.By
import screen.web.demo.shared.DemoBody
import screen.web.demo.shared.DemoPopup
import util.MobileActions

class RelyingPartyMonkeyBikeWebPage : MobileActions() {

    val body = DemoBody()
    val popup = DemoPopup()

    private val loginFailedMessage = By.xpath("//div[@role='alert' and contains(./span/text(), 'Inloggen mislukt. Probeer het opnieuw.')]")

    fun loginFailedMessageVisible() = isWebElementVisible(findWebElement(loginFailedMessage))

    fun openSameDeviceWalletFlow() {
        body.clickStartButton()
        popup.clickSameDeviceButton()
    }
}



