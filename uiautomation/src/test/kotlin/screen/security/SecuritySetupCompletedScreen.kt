package screen.security

import util.MobileActions

class SecuritySetupCompletedScreen : MobileActions() {

    private val setupSecurityCompletedPageTitle = l10n.getString("setupSecurityCompletedPageTitle")
    private val nextButton = l10n.getString("setupSecurityCompletedPageCreateWalletCta")

    fun visible() = elementWithTextVisible(setupSecurityCompletedPageTitle)

    fun absent() = !elementWithTextVisible(setupSecurityCompletedPageTitle)

    fun clickNextButton() = clickElementWithText(nextButton)
}
