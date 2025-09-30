package screen.card

import util.MobileActions

class CardDataScreen : MobileActions() {

    private val dataIncorrectScreenHeaderTitle = l10n.getString("dataIncorrectScreenHeaderTitle")
    private val dataIncorrectButton = l10n.getString("cardDataScreenIncorrectCta")
    private val bottomBackButton = l10n.getString("generalBottomBackCta")

    fun visible() = elementContainingTextVisible(dataIncorrectScreenHeaderTitle)

    fun dataAttributeVisible(attribute: String) = elementContainingTextVisible(attribute)

    fun dataLabelVisible(label: String) = elementContainingTextVisible(label)

    fun dataLabelAbsent(attribute: String) = !elementContainingTextVisible(attribute)

    fun clickDataIncorrectButton() {
        scrollToElementWithText(dataIncorrectButton)
        clickElementWithText(dataIncorrectButton)
    }

    fun clickBottomBackButton() = clickElementWithText(bottomBackButton)
}
