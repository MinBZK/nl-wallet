package nativescreen.web.demo.issuer

import nativescreen.web.demo.shared.DemoBody
import nativescreen.web.demo.shared.DemoPopup
import util.NativeMobileActions

class IssuerWebPage : NativeMobileActions() {

    private val body = DemoBody()
    private val popup = DemoPopup()

    fun openSameDeviceWalletFlow() {
        Thread.sleep(1000)
        body.clickStartButton()
        popup.clickSameDeviceButton()
    }
}
