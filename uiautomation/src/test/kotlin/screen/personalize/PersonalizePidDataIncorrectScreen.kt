package screen.personalize

import util.MobileActions

class PersonalizePidDataIncorrectScreen : MobileActions() {

    private val screen = find.byValueKey("personalizeDataIncorrectScreen")

    fun visible() = isElementVisible(screen)
}
