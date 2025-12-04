package screen.help

import util.MobileActions

class ContactScreen : MobileActions() {

    private val title = l10n.getString("contactScreenTitle")
    private val description = l10n.getString("contactScreenDescription")
    private val callUsTitle = l10n.getString("contactScreenCallUsTitle")
    private val bottomBackButton = l10n.getString("generalBottomBackCta")

    fun visible() = elementWithTextVisible(title) && elementWithTextVisible(description) && elementWithTextVisible(callUsTitle)

    fun clickBottomBackButton() = clickElementWithText(bottomBackButton)
}
