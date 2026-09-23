package screen.issuance

import util.MobileActions

class PersonalizeAuthenticatingWithDigidScreen : MobileActions() {

    private val digidErrorPageTitle = l10n.getString("walletPersonalizeDigidErrorPageTitle")
    private val digidErrorPageLoginWithDigidCta = l10n.getString("walletPersonalizeDigidErrorPageLoginWithDigidCta")
    private val digidErrorPageDigidWebsiteCta = l10n.getString("walletPersonalizeDigidErrorPageDigidWebsiteCta")

    fun loginFailedMessageVisible() = elementWithTextVisible(digidErrorPageTitle)

    fun goToDigiDSiteButtonVisible() = elementWithTextVisible(digidErrorPageDigidWebsiteCta)

    fun tryAgainButtonVisible() = elementWithTextVisible(digidErrorPageLoginWithDigidCta)
}

