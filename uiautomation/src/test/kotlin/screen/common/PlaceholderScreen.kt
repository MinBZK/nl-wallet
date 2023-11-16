package screen.common

import util.MobileActions

class PlaceholderScreen : MobileActions() {

    private val screen = find.byValueKey("placeholderScreen")

    private val backButton = find.byValueKey("introductionBackCta")

    fun visible() = isElementVisible(screen)

    fun clickBackButton() = clickElement(backButton)
}
