package screen.dashboard

import util.MobileActions

class DashboardScreen : MobileActions() {

    private val screen = find.byValueKey("dashboardScreen")

    private val menuButton = find.byText(l10n.getString("dashboardScreenTitle"))

    private val pidIdCard = find.byValueKey("com.example.pid")
    private val pidAddressCard = find.byValueKey("com.example.address")

    private val pidIdTitleText = find.byText(l10n.translate("NL Wallet persoonsgegevens"))
    private val showDetailsText = find.byText(l10n.getString("showDetailsCta"))

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
}
