package nativescreen.web.demo.rp

import org.openqa.selenium.By
import screen.web.demo.shared.DemoBody
import screen.web.demo.shared.DemoPopup
import util.NativeMobileActions

class RelyingPartyXyzBankWebPage : NativeMobileActions() {

    val body = DemoBody()
    val popup = DemoPopup()

    private val accountCreatedTextLocator = By.xpath("//span[text()='Identificatie gelukt']")

    fun identificationSucceededMessageVisible() = isWebElementVisible(findWebElement(accountCreatedTextLocator))

    fun openSameDeviceWalletFlow() {
        body.clickStartButton()
        popup.clickSameDeviceButton()
    }
}
