package screen.revocation

import util.MobileActions

class WalletBlockedByUserScreen : MobileActions() {

    private val title = l10n.getString("appBlockedScreenByUserTitle")
    private val helpdeskButton =l10n.getString("appBlockedScreenHelpdeskCta")

    fun visible() = elementWithTextVisible(title) && elementWithTextVisible(helpdeskButton)
}
