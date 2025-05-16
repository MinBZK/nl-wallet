package screen.dashboard

import util.MobileActions

class DashboardScreen : MobileActions() {

    private val screen = find.byValueKey("dashboardScreen")

    private val menuButton = find.byText(l10n.getString("dashboardScreenTitle"))

    private val pidIdCard = find.byValueKey(cardMetadata.getPidVCT())
    private val pidAddressCard = find.byValueKey(cardMetadata.getAddressVCT())

    private val pidIdTitleText = find.byText(cardMetadata.getPidDisplayName())
    private val pidAddressTitleText = find.byText(cardMetadata.getAddressDisplayName())
    private val showDetailsText = find.byText(l10n.getString("showDetailsCta"))
    private val scanQRButton = find.byText(l10n.getString("menuScreenScanQrCta"))

    fun visible() = isElementVisible(screen, false)

    fun pidCardsVisible(): Boolean {
        return cardVisible(cardMetadata.getPidVCT()) && cardVisible(cardMetadata.getAddressVCT())
    }

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

    fun cardSubtitleVisible(subtitle: String): Boolean {
        return isElementVisible(find.byText(subtitle), false)
    }

    fun openQRScanner() = clickElement(scanQRButton)

    fun cardVisible(vct: String): Boolean {
        scrollToEnd(ScrollableType.CustomScrollView)
        return isElementVisible(find.byValueKey(vct))
    }
}
