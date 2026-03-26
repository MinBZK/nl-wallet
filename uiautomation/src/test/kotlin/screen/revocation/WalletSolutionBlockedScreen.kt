package screen.revocation

import util.MobileActions

class WalletSolutionBlockedScreen : MobileActions() {

    private val title = l10n.getString("appBlockedScreenSolutionRevokedTitle")
    private val moreInfoButton =l10n.getString("appBlockedScreenMoreInfoCta")

    fun visible() = elementWithTextVisible(title) && elementWithTextVisible(moreInfoButton)
}
