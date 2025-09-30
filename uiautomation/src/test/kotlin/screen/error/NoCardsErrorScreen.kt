package screen.error

import util.MobileActions

class NoCardsErrorScreen : MobileActions() {

    private val closeButton = l10n.getString("generalClose")
    private val title = l10n.getString("issuanceNoCardsPageTitle")

    fun titleVisible() = elementWithTextVisible(title)

    fun close() = clickElementWithText(closeButton)
}


