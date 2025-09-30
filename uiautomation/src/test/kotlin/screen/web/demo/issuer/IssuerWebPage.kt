package screen.web.demo.issuer

import screen.web.demo.shared.DemoBody
import screen.web.demo.shared.DemoPopup
import util.MobileActions

class IssuerWebPage : MobileActions() {

    private val body = DemoBody()
    private val popup = DemoPopup()

    fun openSameDeviceWalletFlow() {
        Thread.sleep(1000)
        body.clickStartButton()
        popup.clickSameDeviceButton()
    }
}
