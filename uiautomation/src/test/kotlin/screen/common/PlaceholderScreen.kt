package screen.common

import util.MobileActions

class PlaceholderScreen : MobileActions() {

    private val screen = find.byValueKey("placeholderScreen")

    fun visible() = isElementVisible(screen)
}
