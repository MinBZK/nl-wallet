package screen.security

import util.MobileActions

class ForgotPinScreen : MobileActions() {

    private val title = l10n.getString("forgotPinScreenTitle")
    private val resetWalletButton = l10n.getString("forgotPinScreenResetCta")
    private val bottomBackButton = l10n.getString("generalBottomBackCta")
    private val recoverPinButton = l10n.getString("forgotPinScreenCta")


    fun visible() = elementWithTextVisible(title)

    fun descriptionTextVisible(): Boolean {
        val description = l10n.getString("forgotPinScreenDescription")
        val paragraphed = description.split("\n\n")

        if (paragraphed.isEmpty()) return false

        return paragraphed.all { elementContainingTextVisible(it) }
    }

    fun resetButtonVisible() = elementWithTextVisible(resetWalletButton)

    fun clickBottomBackButton() = clickElementWithText(bottomBackButton)

    fun recoverButtonVisible() = elementWithTextVisible(recoverPinButton)

    fun clickResetPinButton() = clickElementWithText(recoverPinButton)
}
