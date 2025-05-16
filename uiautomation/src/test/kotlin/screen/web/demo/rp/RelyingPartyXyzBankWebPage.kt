package screen.web.demo.rp

import org.openqa.selenium.By
import screen.web.demo.shared.DemoBody
import screen.web.demo.shared.DemoPopup
import util.MobileActions

class RelyingPartyXyzBankWebPage : MobileActions() {

    val body = DemoBody()
    val popup = DemoPopup()

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
