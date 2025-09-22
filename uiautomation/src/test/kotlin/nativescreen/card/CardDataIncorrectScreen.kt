package nativescreen.card

import util.NativeMobileActions

class CardDataIncorrectScreen : NativeMobileActions() {

    private val dataIncorrectScreenHeaderTitle = l10n.getString("dataIncorrectScreenHeaderTitle")
    private val generalBottomBackCta = l10n.getString("generalBottomBackCta")

    fun visible() = elementWithTextVisible(dataIncorrectScreenHeaderTitle)

    fun goBack() = clickElementWithText(generalBottomBackCta)
}
