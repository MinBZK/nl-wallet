package screen.common

import util.MobileActions

class PlaceholderScreen : MobileActions() {

    private val headlineText = find.byText(l10n.getString("placeholderScreenHeadline"))

    fun visible() = isElementVisible(headlineText)
}
