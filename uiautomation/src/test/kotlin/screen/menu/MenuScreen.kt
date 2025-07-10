package screen.menu

import util.MobileActions

class MenuScreen : MobileActions() {

    private val screen = find.byValueKey("menuScreen")

    private val appTourVideoButton = find.byText(l10n.getString("menuScreenTourCta"))
    private val helpButton = find.byText(l10n.getString("menuScreenHelpCta"))
    private val historyButton = find.byText(l10n.getString("menuScreenHistoryCta"))
    private val settingsButton = find.byText(l10n.getString("menuScreenSettingsCta"))
    private val feedbackButton = find.byText(l10n.getString("menuScreenFeedbackCta"))
    private val aboutButton = find.byText(l10n.getString("menuScreenAboutCta"))
    private val logoutButton = find.byText(l10n.getString("menuScreenLockCta"))
    private val bottomBackButton = find.byText(l10n.getString("generalBottomBackCta"))
    private val browserTestButton = find.byText("Browser Test")
    private val scrollableType = ScrollableType.CustomScrollView

    fun visible() = isElementVisible(screen)

    fun menuListButtonsVisible(): Boolean =
        isElementVisible(helpButton) && isElementVisible(historyButton) && isElementVisible(settingsButton) &&
            isElementVisible(feedbackButton) && isElementVisible(aboutButton)

    fun logoutButtonVisible(): Boolean {
        scrollToEnd(scrollableType)
        return isElementVisible(logoutButton)
    }

    fun clickHelpButton() = clickElement(helpButton)

    fun clickHistoryButton() = clickElement(historyButton)

    fun clickSettingsButton() = clickElement(settingsButton)

    fun clickFeedbackButton() = clickElement(feedbackButton)

    fun clickAboutButton() = clickElement(aboutButton)

    fun clickLogoutButton() {
        scrollToEnd(scrollableType)
        clickElement(logoutButton)
    }

    fun clickBottomBackButton() = clickElement(bottomBackButton)

    fun clickBrowserTestButton() {
        scrollToEnd(scrollableType)
        clickElement(browserTestButton)
        switchToWebViewContext()
    }

    fun clickAppTourVideoButton()= clickElement(appTourVideoButton)
}
