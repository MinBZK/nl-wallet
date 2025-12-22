package screen.dashboard

import util.MobileActions

class DashboardScreen : MobileActions() {

    private val menuButton = l10n.getString("dashboardScreenMenuWCAGLabel")
    private val pidIdTitleText = cardMetadata.getPidDisplayName()
    private val showDetailsText = l10n.getString("showDetailsCta")
    private val scanQRButton = l10n.getString("menuScreenScanQrCta")
    private val appTourBannerTitle = l10n.getString("tourBannerTitle")

    fun visible() = elementContainingTextVisible(menuButton) && elementWithTextVisible(scanQRButton)

    fun cardFaceTextsInActiveLanguage() =
        elementContainingTextVisible(pidIdTitleText) && elementContainingTextVisible(showDetailsText)

    fun clickMenuButton() = clickElementContainingText(menuButton)

    fun clickCard(displayName: String) = clickElementContainingText(displayName)

    fun appTourBannerVisible() = elementContainingTextVisible(appTourBannerTitle.substringBefore("'"))

    fun cardTitleVisible() = elementContainingTextVisible(pidIdTitleText)

    fun cardButtonsVisible() = elementContainingTextVisible(showDetailsText)

    fun cardSubtitleVisible(subtitle: String) = elementContainingTextVisible(subtitle)

    fun openQRScanner() = clickElementWithText(scanQRButton)

    fun cardVisible(displayName: String) = elementContainingTextVisible(displayName)
}
