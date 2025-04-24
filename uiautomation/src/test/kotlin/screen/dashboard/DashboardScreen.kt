package screen.dashboard

import util.MobileActions

class DashboardScreen : MobileActions() {

    private val screen = find.byValueKey("dashboardScreen")

    private val menuButton = find.byText(l10n.getString("dashboardScreenTitle"))

    private val pidIdCard = find.byValueKey(cardMetadata.getPidVCT())
    private val pidAddressCard = find.byValueKey(cardMetadata.getAddressACT())

    private val pidIdTitleText = find.byText(cardMetadata.getPidDisplayName())
    private val pidAddressTitleText = find.byText(cardMetadata.getAddressDisplayName())
    private val pidIdSubtitleText = find.byText("")
    private val pidAddressSubtitleText = find.byText("")
    private val showDetailsText = find.byText(l10n.getString("showDetailsCta"))
    private val scanQRButton = find.byText(l10n.getString("menuScreenScanQrCta"))

    fun visible() = isElementVisible(screen, false)

    fun cardsVisible() = isElementVisible(pidIdCard, false) && isElementVisible(pidAddressCard, false)

    fun cardFaceTextsInActiveLanguage() =
        isElementVisible(pidIdTitleText, false) && isElementVisible(showDetailsText, false)

    fun checkCardSorting(): Boolean {
        val (_, pidY) = getTopLeft(pidIdCard, false)!!
        val (_, addressY) = getTopLeft(pidAddressCard, false)!!
        return pidY < addressY
    }

    fun clickMenuButton() = clickElement(menuButton, false)

    fun clickPidCard() {
        scrollToEnd(ScrollableType.CustomScrollView)
        clickElement(pidIdCard, false)
    }

    fun cardTitlesVisible() = isElementVisible(pidIdTitleText, false) && isElementVisible(pidAddressTitleText, false)

    fun cardButtonsVisible() = isElementVisible(find.byDescendant(pidIdCard, showDetailsText, false, false))  && isElementVisible(find.byDescendant(pidAddressCard, showDetailsText, false, false))

    fun cardBackgroundImagesVisible(): Boolean {
        val svg = find.byType("Image")
        return isElementVisible(find.byDescendant(pidIdCard, svg, false, false))   && isElementVisible(find.byAncestor(pidAddressCard, svg, false, false))
    }

    fun cardLogosVisible(): Boolean {
        val png = find.byType("Image")
        return isElementVisible(find.byDescendant(pidIdCard, png, false, false))  && isElementVisible(find.byDescendant(pidAddressCard, png, false, false))
    }

    fun cardSubtitlesVisible() = isElementVisible(pidIdSubtitleText, false) && isElementVisible(pidAddressSubtitleText, false)

    fun openQRScanner() = clickElement(scanQRButton)
}
