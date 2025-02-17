package screen.web.rp.shared

import org.openqa.selenium.By
import util.MobileActions

class RelyingPartyDemoPopup : MobileActions() {

    private val deviceChoiceHeaderDutchTextLocator =
        By.xpath(".//h2[text()='Op welk apparaat staat je NL Wallet app?']")
    private val deviceChoiceHeaderEnglishTextLocator =
        By.xpath(".//h2[text()='Which device is your NL Wallet is installed?']")

    private val scanQrHeaderDutchTextLocator = By.xpath(".//h2[text()='Scan de QR-code met je NL Wallet app']")
    private val sameDeviceButtonLocator = By.xpath(".//*[@data-testid='same_device_button']")
    private val otherDeviceButtonLocator = By.xpath(".//*[@data-testid='cross_device_button']")
    private val qrLocator = By.xpath(".//div[@data-testid='qr']")
    private val helpSectionLocator = By.xpath(".//section[@data-testid='website_link']")
    private val closeButtonLocator = By.xpath(".//button[@data-testid='close_button']")

    fun deviceChoiceDutchTextVisible() =
        isWebElementVisible(getWebModalAnchor().findElement(deviceChoiceHeaderDutchTextLocator))

    fun deviceChoiceEnglishTextVisible() =
        isWebElementVisible(getWebModalAnchor().findElement(deviceChoiceHeaderEnglishTextLocator))

    fun sameDeviceButtonVisible() =
        isWebElementVisible(getWebModalAnchor().findElement(sameDeviceButtonLocator))

    fun otherDeviceButtonVisible() =
        isWebElementVisible(getWebModalAnchor().findElement(otherDeviceButtonLocator))

    fun scanQrTextVisible() =
        isWebElementVisible(getWebModalAnchor().findElement(scanQrHeaderDutchTextLocator))

    fun qrVisible() = isWebElementVisible(getWebModalAnchor().findElement(qrLocator))

    fun helpSectionVisible() = isWebElementVisible(getWebModalAnchor().findElement(helpSectionLocator))
    fun closeButtonVisible() = isWebElementVisible(getWebModalAnchor().findElement(closeButtonLocator))

    fun clickSameDeviceButton() = clickWebElement(getWebModalAnchor().findElement(sameDeviceButtonLocator))

    fun clickOtherDeviceButton() = clickWebElement(getWebModalAnchor().findElement(otherDeviceButtonLocator))

    fun clickCloseButton() = clickWebElement(getWebModalAnchor().findElement(closeButtonLocator))
}
