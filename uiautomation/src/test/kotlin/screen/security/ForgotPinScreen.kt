package screen.security

import util.MobileActions

class ForgotPinScreen : MobileActions() {

    private val screen = find.byValueKey("forgotPinScreen")

    private val resetButton = find.byText(l10n.getString("forgotPinScreenCta"))
    private val bottomBackButton = find.byText(l10n.getString("generalBottomBackCta"))

    fun visible() = isElementVisible(screen)

    fun dataLossTextVisible(): Boolean {
        val description = l10n.getString("forgotPinScreenDescription")
        val paragraphed = description.split("\n\n")

        if (paragraphed.isEmpty()) return false

        return paragraphed.all { isElementVisible(find.byText(it)) }
    }

    fun resetButtonVisible() = isElementVisible(resetButton)

    fun clickBottomBackButton() = clickElement(bottomBackButton)
}
