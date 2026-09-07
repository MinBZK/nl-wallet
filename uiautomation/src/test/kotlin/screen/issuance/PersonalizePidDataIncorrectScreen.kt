package screen.issuance

import util.MobileActions

class PersonalizePidDataIncorrectScreen : MobileActions() {

    private val detailsIncorrectScreenTitle = l10n.getString("detailsIncorrectScreenTitle")
    private val bottomPrimaryButton = l10n.getString("walletPersonalizeDataIncorrectScreenPrimaryCta")
    private val bottomBackButton = l10n.getString("generalBottomBackCta")

    fun visible() = elementWithTextVisible(detailsIncorrectScreenTitle)

    fun clickBottomPrimaryButton() = clickElementWithText(bottomPrimaryButton)

    fun clickBottomBackButton() = clickElementWithText(bottomBackButton)
}
