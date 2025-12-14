package screen.card

import util.MobileActions

class CardDataScreen : MobileActions() {

    private val cardPreviewScreenIncorrectCta = l10n.getString("cardPreviewScreenIncorrectCta")
    private val dataIncorrectButton = l10n.getString("cardDataScreenIncorrectCta")
    private val bottomBackButton = l10n.getString("generalBottomBackCta")

    fun visible(): Boolean {
        scrollToElementWithText(cardPreviewScreenIncorrectCta)
        return elementContainingTextVisible(cardPreviewScreenIncorrectCta)
    }

    fun dataAttributeVisible(attribute: String) = elementContainingTextVisible(attribute)

    fun dataLabelVisible(label: String) = elementContainingTextVisible(label)

    fun dataLabelAbsent(attribute: String) = !elementContainingTextVisible(attribute)

    fun clickDataIncorrectButton() {
        scrollToElementWithText(dataIncorrectButton)
        clickElementWithText(dataIncorrectButton)
    }

    fun clickBottomBackButton() = clickElementWithText(bottomBackButton)
}
