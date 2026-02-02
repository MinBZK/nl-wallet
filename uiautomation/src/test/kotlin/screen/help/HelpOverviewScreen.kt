package screen.help

import util.MobileActions

class HelpOverviewScreen : MobileActions() {

    private val cardsHelpButton = l10n.getString("needHelpScreenCardsTitle")
    private val qrHelpButton = l10n.getString("needHelpScreenQrTitle")
    private val shareDataHelpButton = l10n.getString("needHelpScreenShareTitle")
    private val activitiesHelpButton = l10n.getString("needHelpScreenActivitiesTitle")
    private val securityHelpButton = l10n.getString("needHelpScreenSecurityTitle")
    private val settingsHelpButton = l10n.getString("needHelpScreenSettingsTitle")
    private val contactButton = l10n.getString("needHelpScreenContactTitle")
    private val bottomBackButton = l10n.getString("generalBottomBackCta")

    fun helpButtonsVisible() =
        elementContainingTextVisible(cardsHelpButton) && elementContainingTextVisible(qrHelpButton) && elementContainingTextVisible(shareDataHelpButton) &&
            elementContainingTextVisible(activitiesHelpButton) && elementContainingTextVisible(securityHelpButton) && elementContainingTextVisible(settingsHelpButton)

    fun clickContactButton() {
        scrollToElementContainingText(contactButton)
        clickElementContainingText(contactButton)
    }

    fun clickBottomBackButton() = clickElementWithText(bottomBackButton)
}
