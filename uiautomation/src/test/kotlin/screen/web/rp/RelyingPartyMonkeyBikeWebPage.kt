package screen.web.rp

import org.openqa.selenium.By
import screen.web.rp.shared.RelyingPartyDemoBody
import screen.web.rp.shared.RelyingPartyDemoPopup
import util.MobileActions

class RelyingPartyMonkeyBikeWebPage : MobileActions() {

    val body = RelyingPartyDemoBody()
    val popup = RelyingPartyDemoPopup()

    private val loginFailedMessage = By.xpath("//div[@role='alert' and contains(./span/text(), 'Inloggen mislukt. Probeer het opnieuw.')]")

    fun loginFailedMessageVisible() = isWebElementVisible(findElement(loginFailedMessage))

    fun openSameDeviceWalletFlow(platform: String) {
        when (platform) {
            "ANDROID" -> {
                body.clickStartButton()
                popup.clickSameDeviceButton()
            }
            "IOS" -> {
                tapCoordinates(100, 410)
                tapCoordinates(175, 345)
            }
            else -> {
                throw Exception("Platform $platform is not supported")
            }
        }
    }
}



