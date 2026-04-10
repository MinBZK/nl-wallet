package screen.settings

import org.openqa.selenium.By
import util.MobileActions

class NotificationsScreen : MobileActions() {

    private val screenTitle = l10n.getString("manageNotificationsScreenTitle")
    private val debugScreenButton = l10n.getString("manageNotificationsScreenDebugTitle")
    private val pushSettingSubtitle = l10n.getString("manageNotificationsScreenPushSettingSubtitle")

    fun visible() = elementWithTextVisible(screenTitle)

    fun clickDebugScreenButton() = clickElementContainingText(debugScreenButton)

    fun toggleNotifications() = clickElementContainingText(pushSettingSubtitle)

    fun notificationsToggled(): Boolean = when (platformName()) {
        "ANDROID" -> driver.findElement(By.className("android.widget.Switch")).getAttribute("checked") == "true"
        "IOS" -> driver.findElement(By.className("XCUIElementTypeSwitch")).getAttribute("value") == "1"
        else -> throw IllegalArgumentException("Unsupported platform: ${platformName()}")
    }
}
