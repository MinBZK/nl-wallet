package screen.about

import util.MobileActions

class AboutScreen : MobileActions() {

    private val aboutScreenTitle = l10n.getString("aboutScreenTitle")
    private val backButton = l10n.getString("generalBottomBackCta")

    fun visible() = elementWithTextVisible(aboutScreenTitle)

    fun goBack() = clickElementWithText(backButton)
}
