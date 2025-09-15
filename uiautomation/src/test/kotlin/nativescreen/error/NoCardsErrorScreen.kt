package nativescreen.error

import util.NativeMobileActions

class NoCardsErrorScreen : NativeMobileActions() {

    private val closeButton = l10n.getString("generalClose")
    private val title = l10n.getString("issuanceNoCardsPageTitle")

    fun titleVisible() = elementWithTextVisible(title)

    fun close() = clickElementWithText(closeButton)
}


