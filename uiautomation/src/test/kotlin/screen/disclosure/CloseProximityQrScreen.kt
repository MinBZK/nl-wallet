package screen.disclosure

import util.MobileActions

class CloseProximityQrScreen : MobileActions() {

    private val title = l10n.getString("qrPresentScreenTitle")
    private val centerQrButton = l10n.getString("qrPresentScreenCenterQrCodeCta")
    private val qr = l10n.getString("qrPresentScreenQrWCAGLabel")

    fun visible() = elementWithTextVisible(title)

    fun centerQr() = clickElementWithText(centerQrButton)

    fun getQr(): String = decodeQrFromBytes(takeScreenshotOfElement(qr))
}
