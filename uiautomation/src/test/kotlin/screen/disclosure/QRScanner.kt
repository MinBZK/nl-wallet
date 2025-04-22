package screen.disclosure

import util.MobileActions

class QRScanner : MobileActions() {

    private val scanHint = find.byText(l10n.getString("qrScreenScanHint"))
    private val qrScreenEnableTorchCta = find.byText(l10n.getString("qrScreenEnableTorchCta"))
    private val qrScreenDisableTorchCta = find.byText(l10n.getString("qrScreenDisableTorchCta"))
    private val generalBottomBackCta = find.byText(l10n.getString("generalBottomBackCta"))

    fun visible() = isElementVisible(scanHint)

    fun enableTorch() = clickElement(qrScreenEnableTorchCta)

    fun disableTorch() = clickElement(qrScreenDisableTorchCta)

    fun goBack() = clickElement(generalBottomBackCta)
}
