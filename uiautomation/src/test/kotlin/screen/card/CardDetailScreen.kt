package screen.card

import util.MobileActions

class CardDetailScreen : MobileActions() {

    private val screen = find.byValueKey("cardDetailScreen")

    fun visible() = isElementVisible(screen, false)
}
