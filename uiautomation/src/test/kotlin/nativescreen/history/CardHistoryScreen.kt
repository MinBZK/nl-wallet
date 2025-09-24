package nativescreen.history

import util.NativeMobileActions

class CardHistoryScreen : NativeMobileActions() {

    private val screenTitle = l10n.getString("cardHistoryScreenTitle")

    fun visible() = elementWithTextVisible(screenTitle)
}
