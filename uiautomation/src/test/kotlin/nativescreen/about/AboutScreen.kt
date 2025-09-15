package nativescreen.about

import util.NativeMobileActions

class AboutScreen : NativeMobileActions() {

    private val aboutScreenTitle = l10n.getString("aboutScreenTitle")
    private val backButton = l10n.getString("generalBottomBackCta")

    fun visible() = elementWithTextVisible(aboutScreenTitle)

    fun goBack() = clickElementWithText(backButton)
}
