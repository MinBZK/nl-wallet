package nativescreen.issuance

import util.NativeMobileActions

class PersonalizeAuthenticatingWithDigidScreen : NativeMobileActions() {

    private val digidErrorPageTitle = l10n.getString("walletPersonalizeDigidErrorPageTitle")
    private val digidErrorPageLoginWithDigidCta = l10n.getString("walletPersonalizeDigidErrorPageLoginWithDigidCta")
    private val digidErrorPageDigidWebsiteCta = l10n.getString("walletPersonalizeDigidErrorPageDigidWebsiteCta")
    private val screenAwaitingUserAuthTitle = l10n.getString("walletPersonalizeScreenAwaitingUserAuthTitle")
    private val screenDigidLoadingStopCta = l10n.getString("walletPersonalizeScreenDigidLoadingStopCta")

    fun loginFailedMessageVisible() = elementWithTextVisible(digidErrorPageTitle)

    fun goToDigiDSiteButtonVisible() = elementWithTextVisible(digidErrorPageDigidWebsiteCta)

    fun tryAgainButtonVisible() = elementWithTextVisible(digidErrorPageLoginWithDigidCta)

    fun awaitingUserAuthTitleVisible() = elementWithTextVisible(screenAwaitingUserAuthTitle)

    fun digidLoadingStopCtaVisible() = elementWithTextVisible(screenDigidLoadingStopCta)
}

