package screen.web.demo.issuer

import screen.web.demo.shared.DemoBody
import screen.web.demo.shared.DemoPopup
import util.MobileActions

class IssuerWebPage : MobileActions() {

    private val body = DemoBody()
    private val popup = DemoPopup()

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
