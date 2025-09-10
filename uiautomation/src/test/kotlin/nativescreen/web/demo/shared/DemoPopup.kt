package nativescreen.web.demo.shared

import org.openqa.selenium.By
import util.NativeMobileActions

class DemoPopup : NativeMobileActions() {

    private val sameDeviceButtonLocator = By.xpath(".//*[@data-testid='same_device_button']")

    fun clickSameDeviceButton() = clickWebElement(getWebModalAnchor().findElement(sameDeviceButtonLocator))
}
