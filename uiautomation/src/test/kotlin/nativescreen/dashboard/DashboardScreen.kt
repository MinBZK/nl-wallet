package nativescreen.dashboard

import util.NativeMobileActions

class DashboardScreen : NativeMobileActions() {

    private val menuButton = l10n.getString("dashboardScreenMenuWCAGLabel")
    private val pidIdCard = cardMetadata.getPidVCT()
    private val pidAddressCard = cardMetadata.getAddressVCT()
    private val pidIdTitleText = cardMetadata.getPidDisplayName()
    private val pidAddressTitleText = cardMetadata.getAddressDisplayName()
    private val showDetailsText = l10n.getString("showDetailsCta")
    private val scanQRButton = l10n.getString("menuScreenScanQrCta")
    private val appTourBannerTitle = l10n.getString("tourBannerTitle")

    fun visible() = elementContainingTextVisible(menuButton) && elementWithTextVisible(scanQRButton)

    fun pidCardsVisible() = cardVisible(cardMetadata.getPidVCT()) && cardVisible(cardMetadata.getAddressVCT())

    fun cardFaceTextsInActiveLanguage() =
        elementWithTextVisible(pidIdTitleText) && elementWithTextVisible(showDetailsText)

    fun checkCardSorting(): Boolean {
        val (_, pidY) = getTopLeftOfElementWithText(pidIdCard)!!
        val (_, addressY) = getTopLeftOfElementWithText(pidAddressCard)!!
        return pidY < addressY
    }

    fun clickMenuButton() = clickElementWithText(menuButton)

    fun clickCard(vct: String) {
        scrollToElementWithText(vct)
        clickElementWithText(vct)
    }

    fun appTourBannerVisible() = elementWithTextVisible(appTourBannerTitle)

    fun cardTitlesVisible() = elementWithTextVisible(pidIdTitleText) && elementWithTextVisible(pidAddressTitleText)

    fun cardButtonsVisible() {
        elementWithDescendantAndTextAndVisible(pidIdCard, showDetailsText) &&
            elementWithDescendantAndTextAndVisible(pidAddressCard, showDetailsText)
    }

    fun cardSubtitleVisible(subtitle: String): Boolean {
        return elementWithTextVisible(subtitle)
    }

    fun openQRScanner() = clickElementWithText(scanQRButton)

    private fun cardVisible(vct: String): Boolean {
        scrollToElementWithText(vct)
        return elementWithTextVisible(vct)
    }
}
