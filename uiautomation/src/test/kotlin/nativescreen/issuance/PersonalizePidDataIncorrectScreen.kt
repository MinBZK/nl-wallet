package nativescreen.issuance

import util.NativeMobileActions

class PersonalizePidDataIncorrectScreen : NativeMobileActions() {

    private val dataIncorrectScreenHeaderTitle = l10n.getString("dataIncorrectScreenHeaderTitle")
    private val bottomPrimaryButton = l10n.getString("walletPersonalizeDataIncorrectScreenPrimaryCta")
    private val bottomBackButton = l10n.getString("generalBottomBackCta")

    fun visible() = elementWithTextVisible(dataIncorrectScreenHeaderTitle)

    fun clickBottomPrimaryButton() = clickElementWithText(bottomPrimaryButton)

    fun clickBottomBackButton() = clickElementWithText(bottomBackButton)
}
