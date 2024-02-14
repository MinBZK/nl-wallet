package screen.card

import util.MobileActions

class CardDataScreen : MobileActions() {

    private val screen = find.byValueKey("cardDataScreen")

    fun visible() = isElementVisible(screen)
}
