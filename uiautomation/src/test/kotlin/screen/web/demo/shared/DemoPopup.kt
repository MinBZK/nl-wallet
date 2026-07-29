package screen.web.demo.shared

import util.MobileActions

class DemoPopup : MobileActions() {

    fun clickSameDeviceButton() =
        clickWebShadowButtonWithGesture("same_device_button", "On this device", "Op dit apparaat")

    fun clickCrossDeviceButton() = clickWebShadowButton("cross_device_button")

    fun clickCloseButton() = clickWebShadowButton("close_button")
}
