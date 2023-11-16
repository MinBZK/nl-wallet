package screen.card

import util.MobileActions

class CardOverviewScreen : MobileActions() {

    private val screen = find.byValueKey("cardOverviewScreen")

    fun visible() = isElementVisible(screen, false)
}
