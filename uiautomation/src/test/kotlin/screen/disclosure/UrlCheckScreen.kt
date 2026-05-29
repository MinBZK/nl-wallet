package screen.disclosure

import util.MobileActions

class UrlCheckScreen : MobileActions() {

    private val continueButton = l10n.getString("fraudCheckPageContinueCta")

    fun titleVisible(webUrl: String) : Boolean {
        val normalizedUrl = if (webUrl.endsWith("/")) webUrl else "$webUrl/"
        val selectorText = l10n.getString("fraudCheckPageTitle").replace("{url}", normalizedUrl)
        return elementWithTextVisible(selectorText, 10)
    }

    fun clickContinueButton() = clickElementWithText(continueButton)
}
