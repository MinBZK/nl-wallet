package screen.card

import util.MobileActions

class CardDataIncorrectScreen : MobileActions() {

    private val detailsIncorrectScreenTitle = l10n.getString("detailsIncorrectScreenTitle")
    private val generalBottomBackCta = l10n.getString("generalBottomBackCta")

    fun visible() = elementWithTextVisible(detailsIncorrectScreenTitle)

    fun goBack() = clickElementWithText(generalBottomBackCta)
}
