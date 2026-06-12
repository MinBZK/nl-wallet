package screen.help

import util.MobileActions

class HelpAndInfoScreen : MobileActions() {

    private val appTourVideoButton = l10n.getString("menuScreenTourCta")
    private val contactButton = l10n.getString("contactScreenTitle")
    private val bottomBackButton = l10n.getString("generalBottomBackCta")
    //used this key to prevent hardcoding the values since help related text are not in l10n anymore
    private val activitiesHelpButton = l10n.getString("menuScreenHistoryCta")

    fun clickAppTourVideoButton()= clickElementContainingText(appTourVideoButton.substringBefore("'"))

    fun visible() =
        elementContainingTextVisible(appTourVideoButton)

    fun clickContactButton() {
        scrollToElementWithText(contactButton)
        clickElementWithText(contactButton)
    }

    fun clickActivitiesHelpButton() = clickElementContainingText(activitiesHelpButton)

    fun clickBottomBackButton() = clickElementWithText(bottomBackButton)

}
