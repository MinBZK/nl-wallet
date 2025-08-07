package screen.error

import util.MobileActions

class NoCardsErrorScreen : MobileActions() {

    private val closeButton = find.byText(l10n.getString("generalClose"))
    private val title = find.byText(l10n.getString("issuanceNoCardsPageTitle"))


    fun titleVisible() = isElementVisible(title)

    fun close() = clickElement(closeButton, false)
}


