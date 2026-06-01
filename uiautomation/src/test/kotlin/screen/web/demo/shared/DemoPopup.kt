package screen.web.demo.shared

import org.openqa.selenium.By
import util.MobileActions

class DemoPopup : MobileActions() {

    private val sameDeviceButtonLocator = By.xpath(".//*[@data-testid='same_device_button']")
    private val crossDeviceButtonLocator = By.xpath(".//*[@data-testid='cross_device_button']")
    private val closeButton = By.xpath(".//*[@data-testid='close_button']")

    fun clickSameDeviceButton() = clickWebElement(getWebModalAnchor().findElement(sameDeviceButtonLocator))

    fun clickCrossDeviceButton() = clickWebElement(getWebModalAnchor().findElement(crossDeviceButtonLocator))

    fun clickCloseButton() = clickWebElement(getWebModalAnchor().findElement(closeButton))
}
