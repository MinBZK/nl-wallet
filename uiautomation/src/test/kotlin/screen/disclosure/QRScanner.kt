package screen.disclosure

import util.MobileActions

class QRScanner : MobileActions() {

    private val scanHint = l10n.getString("qrScreenScanHint")
    private val qrScreenEnableTorchCta = l10n.getString("qrScreenEnableTorchCta")
    private val qrScreenDisableTorchCta = l10n.getString("qrScreenDisableTorchCta")
    private val generalBottomBackCta = l10n.getString("generalBottomBackCta")
    private val permissionHint = l10n.getString("qrScreenPermissionHint")
    private val grantPermissionCta = l10n.getString("qrScanTabGrantPermissionCta")

    fun visible() = elementWithTextVisible(scanHint)

    fun enableTorch() = clickElementWithText(qrScreenEnableTorchCta)

    fun disableTorch() = clickElementWithText(qrScreenDisableTorchCta)

    fun goBack() = clickElementWithText(generalBottomBackCta)

    fun permissionHintVisible() = elementWithTextVisible(permissionHint)

    fun grantPermissionButtonVisible() = elementWithTextVisible(grantPermissionCta)
}
