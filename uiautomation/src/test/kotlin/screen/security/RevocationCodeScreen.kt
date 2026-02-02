package screen.security

import util.MobileActions

class RevocationCodeScreen : MobileActions() {

    private val title = l10n.getString("revocationCodeScreenTitle")
    private val confirmButton =l10n.getString("revocationCodeScreenContinueCta")
    private val revocationCodeSelector = "-\n-\n-\n-"

    fun visible() = elementWithTextVisible(title)

    fun confirmReceival() = clickElementWithText(confirmButton)

    fun getRevocationCode(): String {
        return getTextFromAllChildElementsFromElementWithText(revocationCodeSelector).replace(revocationCodeSelector, "")
    }
}
