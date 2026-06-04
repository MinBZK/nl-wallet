package screen.web.demo.issuer

import screen.web.demo.shared.DemoBody
import screen.web.demo.shared.DemoPopup
import util.MobileActions

class IssuerWebPage : MobileActions() {

    private val body = DemoBody()
    private val popup = DemoPopup()

    fun openSameDeviceWalletFlow() {
        Thread.sleep(SET_FRAME_SYNC_MAX_WAIT_MILLIS)
        body.clickStartButton()
        popup.clickSameDeviceButton()
    }

    fun openCrossDeviceWalletFlow() {
        Thread.sleep(SET_FRAME_SYNC_MAX_WAIT_MILLIS)
        body.clickStartButton()
        popup.clickCrossDeviceButton()
    }
}
