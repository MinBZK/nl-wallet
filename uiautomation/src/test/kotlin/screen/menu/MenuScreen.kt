package screen.menu

import util.MobileActions

class MenuScreen : MobileActions() {

    private val appTourVideoButton = l10n.getString("menuScreenTourCta")
    private val helpButton = l10n.getString("menuScreenHelpCta")
    private val historyButton = l10n.getString("menuScreenHistoryCta")
    private val settingsButton = l10n.getString("menuScreenSettingsCta")
    private val feedbackButton = l10n.getString("menuScreenFeedbackCta")
    private val aboutButton = l10n.getString("menuScreenAboutCta")
    private val logoutButton = l10n.getString("menuScreenLockCta")
    private val bottomBackButton = l10n.getString("generalBottomBackCta")
    private val browserTestButton = "Browser Test"

    fun menuListButtonsVisible() =
        elementWithTextVisible(helpButton) && elementWithTextVisible(historyButton) && elementWithTextVisible(settingsButton) &&
            elementWithTextVisible(feedbackButton) && elementWithTextVisible(aboutButton)

    fun logoutButtonVisible(): Boolean {
        scrollToElementWithText(logoutButton)
        return elementWithTextVisible(logoutButton)
    }

    fun clickHelpButton() = clickElementWithText(helpButton)

    fun clickHistoryButton() = clickElementWithText(historyButton)

    fun clickSettingsButton() = clickElementWithText(settingsButton)

    fun clickFeedbackButton() = clickElementWithText(feedbackButton)

    fun clickAboutButton() {
        scrollToElementWithText(aboutButton)
        clickElementWithText(aboutButton)
    }

    fun clickLogoutButton() {
        scrollToElementWithText(logoutButton)
        clickElementWithText(logoutButton)
    }

    fun clickBottomBackButton() = clickElementWithText(bottomBackButton)

    fun clickBrowserTestButton() {
        scrollToElementWithText(logoutButton)
        clickElementContainingText(browserTestButton)
    }

    fun clickAppTourVideoButton()= clickElementContainingText(appTourVideoButton.substringBefore("'"))
}
