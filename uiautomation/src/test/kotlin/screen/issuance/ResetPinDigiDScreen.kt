package screen.issuance

import util.MobileActions

class ResetPinDigiDScreen : MobileActions() {

    private val walletPersonalizeIntroPageTitle = l10n.getString("walletPersonalizeIntroPageTitle")
    private val digidLoginButton = l10n.getString("walletPersonalizeIntroPageLoginWithDigidCta")

    fun visible() = elementWithTextVisible(walletPersonalizeIntroPageTitle)

    fun clickDigidLoginButton() = clickElementWithText(digidLoginButton)
}
