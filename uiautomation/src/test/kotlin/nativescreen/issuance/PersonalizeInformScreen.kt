package nativescreen.issuance

import util.NativeMobileActions

class PersonalizeInformScreen : NativeMobileActions() {

    private val walletPersonalizeIntroPageTitle = l10n.getString("walletPersonalizeIntroPageTitle")
    private val digidLoginButton = l10n.getString("walletPersonalizeIntroPageLoginWithDigidCta")
    private val digidWebsiteButton = l10n.getString("walletPersonalizeIntroPageDigidWebsiteCta")

    fun visible() = elementWithTextVisible(walletPersonalizeIntroPageTitle)

    fun clickDigidLoginButton() = clickElementWithText(digidLoginButton)

    fun digidWebsiteButtonVisible() = elementWithTextVisible(digidWebsiteButton)
}
