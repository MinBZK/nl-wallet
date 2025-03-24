package screen.web.rp.shared

import org.openqa.selenium.By
import util.MobileActions

class RelyingPartyDemoPopup : MobileActions() {

    private val sameDeviceButtonLocator = By.xpath(".//*[@data-testid='same_device_button']")

    fun clickSameDeviceButton() = clickWebElement(getWebModalAnchor().findElement(sameDeviceButtonLocator))
}
