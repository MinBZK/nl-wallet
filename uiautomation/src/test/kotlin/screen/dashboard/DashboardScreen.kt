package screen.dashboard

import util.MobileActions

class DashboardScreen : MobileActions() {

    private val menuButton = l10n.getString("dashboardScreenMenuWCAGLabel")
    private val pidIdTitleText = cardMetadata.getPidDisplayName()
    private val showDetailsText = l10n.getString("showDetailsCta")
    private val actionButton = l10n.getString("qrActionButtonTitle")
    private val appTourBannerTitle = l10n.getString("tourBannerTitle")
    private val revokedLabel = l10n.getString("cardStatusMetadataWalletItemRevoked")
    private val cardRevocationBannerTitle = l10n.getString("cardRevocationBannerTitle")
    private val actionDrawerScanQrButton = l10n.getString("qrActionSheetScanQrTitle")
    private val actionDrawerShowQrButton = l10n.getString("qrActionSheetShowQrTitle")
    private val activitiesButtonTitle = l10n.getString("activitySummaryToday")
    private val requestNotificationPermissionSheetNegativeCta = l10n.getString("requestNotificationPermissionSheetNegativeCta")

    fun visible() = elementContainingTextVisible(menuButton) && elementContainingTextVisible(actionButton)

    fun cardFaceTextsInActiveLanguage() =
        elementContainingTextVisible(pidIdTitleText) && elementContainingTextVisible(showDetailsText)

    fun clickMenuButton() = clickElementContainingText(menuButton)

    // Sometimes the display name of a card is present in the activities button there for it is needed to
    // click an element that does not contain the activities button title. The card is scrolled into view first,
    // so that it does not depend on where an earlier step left the dashboard.
    fun clickCard(displayName: String) {
        Thread.sleep(SCREEN_TRANSITION_MILLIS)
        scrollToElementContainingTextExcludingText(displayName, activitiesButtonTitle).click()
    }

    fun appTourBannerVisible() = elementContainingTextVisible(appTourBannerTitle.substringBefore("'"))

    fun cardTitleVisible() = elementContainingTextVisible(pidIdTitleText)

    fun cardButtonsVisible() = elementContainingTextVisible(showDetailsText)

    fun cardSubtitleVisible(subtitle: String) = elementContainingTextVisible(subtitle)

    fun openQRScanner() {
        clickElementContainingText(actionButton)
        clickElementContainingText(actionDrawerScanQrButton)
    }

    fun showQRCode() {
        clickElementContainingText(actionButton)
        clickElementContainingText(actionDrawerShowQrButton)
    }

    // Sometimes the display name of a card is present in the activities button there for it is needed to
    // click an element that does not contain the activities button title
    fun cardVisible(cardDisplayContent: String): Boolean = try {
        scrollToElementContainingTextExcludingText(cardDisplayContent, activitiesButtonTitle)
        elementContainingTextExcludingTextVisible(cardDisplayContent, activitiesButtonTitle)
    } catch (e: NoSuchElementException) {
        false
    }

    fun cardRevocationVisible(cardDisplayContent: String): Boolean {
        scrollToElementContainingTexts(listOf(cardDisplayContent, revokedLabel))
        return elementContainingTextsVisible(listOf(cardDisplayContent, revokedLabel))
    }

    fun checkCardSorting(topCardDisplayName: String, bottomCardDisplayName: String): Boolean {
        scrollToElementContainingText(topCardDisplayName)
        val (_, pidY) = getTopLeftOfElementContainingText(topCardDisplayName)!!
        scrollToElementContainingText(bottomCardDisplayName)
        val (_, addressY) = getTopLeftOfElementContainingText(bottomCardDisplayName)!!
        return pidY < addressY
    }

    fun tapRevocationNotification(cardDisplayName: String) {
        scrollToElementContainingText(cardRevocationBannerTitle.replace("{card}", cardDisplayName))
        clickElementContainingText(cardRevocationBannerTitle.replace("{card}", cardDisplayName))
    }

    // TODO: Remove conditions for click after PVW-6367
    fun dismissNotificationsDrawer() {
        if (elementWithTextVisible(requestNotificationPermissionSheetNegativeCta, 5)) {
            clickElementContainingText(requestNotificationPermissionSheetNegativeCta)
        }
    }
}
