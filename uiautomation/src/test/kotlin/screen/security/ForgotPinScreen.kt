package screen.security

import util.MobileActions

class ForgotPinScreen : MobileActions() {

    private val screen = find.byValueKey("forgotPinScreen")

    private val dataLossText = find.byText(l10n.getString("forgotPinScreenDescription"))

    private val resetButton = find.byText(l10n.getString("forgotPinScreenCta"))
    private val bottomBackButton = find.byText(l10n.getString("generalBottomBackCta"))

    fun visible() = isElementVisible(screen)

    fun dataLossTextVisible() = isElementVisible(dataLossText)

    fun resetButtonVisible() = isElementVisible(resetButton)

    fun clickBottomBackButton() = clickElement(bottomBackButton)
}
