package screen.card

import util.MobileActions

class CardHistoryScreen : MobileActions() {

    private val screen = find.byValueKey("cardHistoryScreen")

    fun visible() = isElementVisible(screen, false)
}
