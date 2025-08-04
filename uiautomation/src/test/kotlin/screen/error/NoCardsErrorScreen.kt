package screen.error

import util.MobileActions

class NoCardsErrorScreen : MobileActions() {

    private val closeButton = find.byText(l10n.getString("generalClose"))

    fun headlineVisible(organization: String): Boolean {
        return isElementVisible(find.byText(l10n.getString("issuanceNoCardsPageDescription").replace("{organization}", organization)), false)
    }

    fun close() = clickElement(closeButton, false)
}
