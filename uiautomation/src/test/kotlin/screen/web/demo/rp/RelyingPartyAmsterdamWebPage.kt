package screen.web.demo.rp

import org.openqa.selenium.By
import screen.web.demo.shared.DemoBody
import screen.web.demo.shared.DemoPopup
import util.MobileActions

class RelyingPartyAmsterdamWebPage : MobileActions() {

    val body = DemoBody()
    val popup = DemoPopup()

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
