package screen.web.demo.rp

import org.openqa.selenium.By
import screen.web.demo.shared.DemoBody
import screen.web.demo.shared.DemoPopup
import util.MobileActions

class RelyingPartyXyzBankWebPage : MobileActions() {

    val body = DemoBody()
    val popup = DemoPopup()

    fun sharedAttributeVisible(attributeValue: String) = isWebElementVisible(findWebElement(By.xpath("//dd[normalize-space(text())='$attributeValue']")))

    fun openSameDeviceWalletFlow() {
        body.clickStartButton()
        popup.clickSameDeviceButton()
    }
}
