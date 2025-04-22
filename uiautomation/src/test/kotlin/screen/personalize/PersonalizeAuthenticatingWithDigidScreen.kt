package screen.personalize

import util.MobileActions

class PersonalizeAuthenticatingWithDigidScreen : MobileActions() {

    private val screen = find.byValueKey("personalizeAuthenticatingWithDigidPage")
    private val walletPersonalizeDigidErrorPageTitle = find.byText(l10n.getString("walletPersonalizeDigidErrorPageTitle"))
    private val walletPersonalizeDigidErrorPageLoginWithDigidCta = find.byText(l10n.getString("walletPersonalizeDigidErrorPageLoginWithDigidCta"))
    private val walletPersonalizeDigidErrorPageDigidWebsiteCta = find.byText(l10n.getString("walletPersonalizeDigidErrorPageDigidWebsiteCta"))
    private val walletPersonalizeScreenAwaitingUserAuthTitle = find.byText(l10n.getString("walletPersonalizeScreenAwaitingUserAuthTitle"))
    private val walletPersonalizeScreenDigidLoadingStopCta = find.byText(l10n.getString("walletPersonalizeScreenDigidLoadingStopCta"))

    fun visible() = isElementVisible(screen, false)

    fun loginFailedMessageVisible() = isElementVisible(walletPersonalizeDigidErrorPageTitle)

    fun goToDigiDSiteButtonVisible() = isElementVisible(walletPersonalizeDigidErrorPageDigidWebsiteCta)

    fun tryAgainButtonVisible() = isElementVisible(walletPersonalizeDigidErrorPageLoginWithDigidCta)

    fun awaitingUserAuthTitleVisible() = isElementVisible(walletPersonalizeScreenAwaitingUserAuthTitle)

    fun digidLoadingStopCtaVisible() = isElementVisible(walletPersonalizeScreenDigidLoadingStopCta)
}

