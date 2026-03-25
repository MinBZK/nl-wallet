package screen.revocation

import util.MobileActions

class WalletBlockedByWalletIdScreen : MobileActions() {

    private val title = l10n.getString("appBlockedScreenTitle")
    private val helpdeskButton =l10n.getString("appBlockedScreenHelpdeskCta")
    private val createNewWalletButton =l10n.getString("appBlockedScreenNewWalletCta")

    fun visible() = elementWithTextVisible(title) && elementWithTextVisible(helpdeskButton) && elementWithTextVisible(createNewWalletButton)
}
