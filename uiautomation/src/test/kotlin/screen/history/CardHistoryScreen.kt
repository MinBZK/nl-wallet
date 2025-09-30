package screen.history

import util.MobileActions

class CardHistoryScreen : MobileActions() {

    private val screenTitle = l10n.getString("cardHistoryScreenTitle")

    fun visible() = elementWithTextVisible(screenTitle)
}
