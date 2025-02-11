package screen.web.rp

import org.openqa.selenium.By
import screen.web.rp.shared.RelyingPartyDemoBody
import screen.web.rp.shared.RelyingPartyDemoPopup
import util.MobileActions

class RelyingPartyXyzBankWebPage : MobileActions() {

    val body = RelyingPartyDemoBody()
    val popup = RelyingPartyDemoPopup()

    private val accountCreatedTextLocator = By.xpath("//span[text()='Identificatie gelukt']")

    fun identificationSucceededMessageVisible() = isWebElementVisible(findElement(accountCreatedTextLocator))

    fun openSameDeviceWalletFlow(platform: String) {
        when (platform) {
            "ANDROID" -> {
                body.clickStartButton()
                popup.clickSameDeviceButton()
            }
            "IOS" -> {
                tapCoordinates(100, 410)
                tapCoordinates(175, 340)
            }
            else -> {
                throw Exception("Platform $platform is not supported")
            }
        }
    }
}
