package screen.disclosure

import util.MobileActions

class SharingStoppedScreen : MobileActions() {

    private val title = l10n.getString("issuanceStoppedPageTitle")
    private val description = l10n.getString("issuanceStoppedPageDescription")
    private val closeButton = l10n.getString("issuanceStoppedPageCloseCta")

    fun titleVisible() = elementWithTextVisible(title)
    fun descriptionVisible() = elementWithTextVisible(description)

    fun close() = clickElementWithText(closeButton)
}
