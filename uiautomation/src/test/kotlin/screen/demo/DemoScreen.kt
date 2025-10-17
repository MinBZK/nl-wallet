package screen.demo

import util.MobileActions

class DemoScreen : MobileActions() {

    private val demoScreenTitle = l10n.getString("demoScreenTitle")
    private val continueButton = l10n.getString("demoScreenContinueCta")

    fun visible() = elementWithTextVisible(demoScreenTitle)

    fun clickContinueButton() = clickElementWithText(continueButton)
}
