package screen.card

import util.MobileActions

class CardDataIncorrectScreen : MobileActions() {

    private val screen = find.byValueKey("cardDataIncorrectScreen")

    fun visible() = isElementVisible(screen)
}
