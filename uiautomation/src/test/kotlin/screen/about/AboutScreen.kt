package screen.about

import util.MobileActions

class AboutScreen : MobileActions() {

    private val screen = find.byValueKey("aboutScreen")

    fun visible() = isElementVisible(screen)
}
