package screen.web.rp

import org.openqa.selenium.By
import screen.web.rp.shared.RelyingPartyDemoBody
import screen.web.rp.shared.RelyingPartyDemoPopup
import util.MobileActions

class RelyingPartyAmsterdamWebPage : MobileActions() {

    val body = RelyingPartyDemoBody()
    val popup = RelyingPartyDemoPopup()

    private val loggedInHeaderTextDutchLocator = By.xpath("//h2[text()='Welkom in Mijn Amsterdam']")

    fun loggedInMessageVisible() = isWebElementVisible(findElement(loggedInHeaderTextDutchLocator))

    fun openSameDeviceWalletFlow(platform: String) {
        when (platform) {
            "ANDROID" -> {
                body.clickStartButton()
                popup.clickSameDeviceButton()
            }
            "IOS" -> {
                tapCoordinates(100, 500)
                tapCoordinates(175, 340)
            }
            else -> {
                throw Exception("Platform $platform is not supported")
            }
        }
    }
}
