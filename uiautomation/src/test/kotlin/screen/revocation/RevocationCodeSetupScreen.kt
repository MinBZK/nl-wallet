package screen.revocation

import util.MobileActions

class RevocationCodeSetupScreen : MobileActions() {

    private val title = l10n.getString("revocationCodeScreenTitle")
    private val confirmButton =l10n.getString("revocationCodeScreenContinueCta")
    private val revocationCodeSelector = "-\n-\n-\n-"

    fun visible() = elementWithTextVisible(title)

    fun confirmReceive() = clickElementWithText(confirmButton)

    fun getRevocationCode(): String {
        return getTextFromAllChildElementsFromElementWithText(revocationCodeSelector).replace(revocationCodeSelector, "")
    }
}
