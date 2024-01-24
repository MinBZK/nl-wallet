package screen.personalize

import util.MobileActions

class PersonalizePidDataIncorrectScreen : MobileActions() {

    private val screen = find.byValueKey("personalizeDataIncorrectScreen")

    private val bottomPrimaryButton = find.byText(l10n.getString("walletPersonalizeDataIncorrectScreenPrimaryCta"))
    private val bottomBackButton = find.byText(l10n.getString("generalBottomBackCta"))

    fun visible() = isElementVisible(screen)

    fun clickBottomPrimaryButton() = clickElement(bottomPrimaryButton)

    fun clickBottomBackButton() = clickElement(bottomBackButton)
}
