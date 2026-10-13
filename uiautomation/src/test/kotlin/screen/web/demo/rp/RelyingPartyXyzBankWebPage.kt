package screen.web.demo.rp

import org.openqa.selenium.By
import screen.web.demo.shared.DemoBody
import screen.web.demo.shared.DemoPopup
import util.MobileActions

class RelyingPartyXyzBankWebPage : MobileActions() {

    val body = DemoBody()
    val popup = DemoPopup()

    fun sharedAttributeVisible(attributeValue: String): Boolean {
        val locator = By.xpath("//dd[normalize-space(text())='$attributeValue']")
        switchToWebViewWindowContaining(locator)
        return isWebElementVisible(findWebElement(locator))
    }

    fun clickCloseButton() = popup.clickCloseButton()

    fun openSameDeviceWalletFlow() {
        body.clickStartButton()
        popup.clickSameDeviceButton()
    }

    fun openCrossDeviceWalletFlow() {
        body.clickStartButton()
        popup.clickCrossDeviceButton()
    }

}
