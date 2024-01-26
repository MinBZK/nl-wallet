package screen.home

import util.MobileActions

open class HomeScreen : MobileActions() {

    private val menuButton = find.byToolTip(l10n.getString("homeScreenBottomNavBarMenuCta"))

    fun clickMenuButton() = clickElement(menuButton, false)
}
