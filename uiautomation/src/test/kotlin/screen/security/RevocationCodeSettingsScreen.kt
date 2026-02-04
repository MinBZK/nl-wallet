package screen.security

import util.MobileActions

class RevocationCodeSettingsScreen : MobileActions() {

    private val title = l10n.getString("reviewRevocationCodeScreenSuccessTitle")
    private val viewButton =l10n.getString("reviewRevocationCodeScreenViewCta")
    private val revocationCodeSelector = "-\n-\n-\n-"

    fun visible() = elementWithTextVisible(title)

    fun clickViewButton() = clickElementWithText(viewButton)

    fun getRevocationCode(): String {
        return getTextFromAllChildElementsFromElementWithText(revocationCodeSelector).replace(revocationCodeSelector, "")
    }
}
