package screen.issuance

import util.MobileActions

class PersonalizePidDataIncorrectScreen : MobileActions() {

    private val dataIncorrectScreenHeaderTitle = l10n.getString("dataIncorrectScreenHeaderTitle")
    private val bottomPrimaryButton = l10n.getString("walletPersonalizeDataIncorrectScreenPrimaryCta")
    private val bottomBackButton = l10n.getString("generalBottomBackCta")

    fun visible() = elementWithTextVisible(dataIncorrectScreenHeaderTitle)

    fun clickBottomPrimaryButton() = clickElementWithText(bottomPrimaryButton)

    fun clickBottomBackButton() = clickElementWithText(bottomBackButton)
}
