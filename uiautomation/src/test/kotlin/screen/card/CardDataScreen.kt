package screen.card

import util.MobileActions

class CardDataScreen : MobileActions() {

    private val screen = find.byValueKey("cardDataScreen")
    private val dataPrivacyBanner = find.byValueKey("dataPrivacyBanner")
    private val dataIncorrectButton = find.byText(l10n.getString("cardDataScreenIncorrectCta"))
    private val bottomBackButton = find.byText(l10n.getString("generalBottomBackCta"))

    private val scrollableType = ScrollableType.CustomScrollView

    fun visible() = isElementVisible(screen)

    fun dataPrivacyBannerVisible() = isElementVisible(dataPrivacyBanner)

    fun dataAttributeVisible(attribute: String): Boolean {
        return isElementVisible(find.byText(attribute))
    }

    fun dataLabelVisible(attribute: String): Boolean {
        return isElementVisible(find.byText(attribute))
    }

    fun clickDataIncorrectButton() = clickElement(dataIncorrectButton)

    fun clickBottomBackButton() = clickElement(bottomBackButton)

    fun scrollToEnd() = scrollToEnd(scrollableType)
}
