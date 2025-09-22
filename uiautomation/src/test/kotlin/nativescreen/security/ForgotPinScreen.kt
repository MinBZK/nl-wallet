package nativescreen.security

import util.NativeMobileActions

class ForgotPinScreen : NativeMobileActions() {

    private val title = l10n.getString("forgotPinScreenTitle")
    private val resetButton = l10n.getString("forgotPinScreenResetCta")
    private val bottomBackButton = l10n.getString("generalBottomBackCta")

    fun visible() = elementWithTextVisible(title)

    fun dataLossTextVisible(): Boolean {
        val description = l10n.getString("forgotPinScreenResetDescription")
        val paragraphed = description.split("\n\n")

        if (paragraphed.isEmpty()) return false

        return paragraphed.all { elementContainingTextVisible(it) }
    }

    fun resetButtonVisible() = elementWithTextVisible(resetButton)

    fun clickBottomBackButton() = clickElementWithText(bottomBackButton)
}
