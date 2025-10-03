package screen.disclosure

import util.MobileActions

class QRScanner : MobileActions() {

    private val scanHint = l10n.getString("qrScreenScanHint")
    private val qrScreenEnableTorchCta = l10n.getString("qrScreenEnableTorchCta")
    private val qrScreenDisableTorchCta = l10n.getString("qrScreenDisableTorchCta")
    private val generalBottomBackCta = l10n.getString("generalBottomBackCta")

    fun visible() = elementWithTextVisible(scanHint)

    fun enableTorch() = clickElementWithText(qrScreenEnableTorchCta)

    fun disableTorch() = clickElementWithText(qrScreenDisableTorchCta)

    fun goBack() = clickElementWithText(generalBottomBackCta)
}
