package screen.card

import util.MobileActions

class CardDeletedScreen : MobileActions() {

    private val cardDeletedDescription = l10n.getString("deleteCardSuccessPageDescription")
    private val toDashboardCta = l10n.getString("deleteCardSuccessPageToDashboardCta")

    fun visible() = elementWithTextVisible(cardDeletedDescription)

    fun clickToDashboardButton() = clickElementWithText(toDashboardCta)
}
