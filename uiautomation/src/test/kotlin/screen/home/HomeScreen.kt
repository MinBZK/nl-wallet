package screen.home

import util.MobileActions

open class HomeScreen : MobileActions() {

    private val cardsButton = find.byText(l10n.getString("homeScreenBottomNavBarCardsCta"))
    private val qrButton = find.byText(l10n.getString("homeScreenBottomNavBarQrCta"))
    private val menuButton = find.byText(l10n.getString("homeScreenBottomNavBarMenuCta"))

    fun clickCardsButton() = clickElement(cardsButton, false)

    fun clickQrButton() = clickElement(qrButton, false)

    fun clickMenuButton() = clickElement(menuButton, false)
}
