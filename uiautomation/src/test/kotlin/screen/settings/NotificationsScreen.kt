package screen.settings

import domain.Platform
import org.openqa.selenium.By
import util.MobileActions

class NotificationsScreen : MobileActions() {

    private val screenTitle = l10n.getString("manageNotificationsScreenTitle")
    private val debugScreenButton = l10n.getString("manageNotificationsScreenDebugTitle")
    private val pushSettingSubtitle = l10n.getString("manageNotificationsScreenPushSettingSubtitle")

    fun visible() = elementWithTextVisible(screenTitle)

    fun clickDebugScreenButton() = clickElementContainingText(debugScreenButton)

    fun toggleNotifications() = clickElementContainingText(pushSettingSubtitle)

    fun notificationsToggled(): Boolean = when (platform()) {
        Platform.ANDROID -> driver.findElement(By.className("android.widget.Switch")).getAttribute("checked") == "true"
        Platform.IOS -> driver.findElement(By.className("XCUIElementTypeSwitch")).getAttribute("value") == "1"
    }
}
