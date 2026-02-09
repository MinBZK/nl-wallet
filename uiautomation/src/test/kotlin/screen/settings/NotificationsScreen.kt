package screen.settings

import util.MobileActions

class NotificationsScreen : MobileActions() {

    private val screenTitle = l10n.getString("manageNotificationsScreenTitle")
    private val debugScreenButton = l10n.getString("manageNotificationsScreenDebugTitle")
    private val pushSettingSubtitle = l10n.getString("manageNotificationsScreenPushSettingSubtitle")

    fun visible() = elementWithTextVisible(screenTitle)

    fun clickDebugScreenButton() = clickElementContainingText(debugScreenButton)

    fun toggleNotifications() = clickElementContainingText(pushSettingSubtitle)
}
