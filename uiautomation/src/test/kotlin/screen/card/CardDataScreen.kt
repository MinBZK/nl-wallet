package screen.card

import util.MobileActions

class CardDataScreen : MobileActions() {

    private val screen = find.byValueKey("cardDataScreen")
    private val dataIncorrectButton = find.byText(l10n.getString("cardDataScreenIncorrectCta"))
    private val bottomBackButton = find.byText(l10n.getString("generalBottomBackCta"))

    private val scrollableElement = find.byType(ScrollableType.CustomScrollView.toString())

    fun visible() = isElementVisible(screen)

    fun dataAttributeVisible(attribute: String): Boolean {
        return isElementVisible(find.byText(attribute))
    }

    fun dataLabelVisible(attribute: String): Boolean {
        return isElementVisible(find.byText(attribute))
    }

    fun dataLabelAbsent(attribute: String): Boolean {
        return isElementAbsent(find.byText(attribute))
    }

    fun clickDataIncorrectButton() = clickElement(dataIncorrectButton)

    fun clickBottomBackButton() = clickElement(bottomBackButton)

    fun scrollToEnd() = scrollToEnd(scrollableElement)
}
