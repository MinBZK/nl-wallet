package screen.revocation

import util.MobileActions

class WalletBlockedByRecoveryCodeScreen : MobileActions() {

    private val title = l10n.getString("appBlockedScreenPermanentTitle")
    private val helpdeskButton =l10n.getString("appBlockedScreenHelpdeskCta")

    fun visible() = elementWithTextVisible(title) && elementWithTextVisible(helpdeskButton)
}
