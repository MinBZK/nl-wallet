package screen.permissions

import org.openqa.selenium.By
import org.openqa.selenium.NoAlertPresentException
import util.MobileActions

class NativePermissionDialog : MobileActions() {

    private val androidDenyButtonId = "com.android.permissioncontroller:id/permission_deny_button"
    private val androidDenyDontAskAgainButtonId = "com.android.permissioncontroller:id/permission_deny_and_dont_ask_again_button"
    private val androidAllowOneTimeButtonId = "com.android.permissioncontroller:id/permission_allow_one_time_button"

    fun visible(): Boolean = when (platformName()) {
        "ANDROID" -> driver.findElements(By.id(androidDenyButtonId)).isNotEmpty()
        "IOS" -> try { driver.switchTo().alert(); true } catch (_: NoAlertPresentException) { false }
        else -> false
    }

    fun deny() = when (platformName()) {
        "ANDROID" -> clickWebElement(findWebElement(By.id(androidDenyButtonId)))
        "IOS" -> driver.switchTo().alert().dismiss()
        else -> throw IllegalStateException("Unsupported platform: ${platformName()}")
    }

    fun denyDontAskAgain() = when (platformName()) {
        "ANDROID" -> clickWebElement(findWebElement(By.id(androidDenyDontAskAgainButtonId)))
        else -> throw IllegalStateException("Unsupported platform: ${platformName()}")
    }

    fun allowOneTimeOnly() = when (platformName()) {
        "ANDROID" -> clickWebElement(findWebElement(By.id(androidAllowOneTimeButtonId)))
        else -> throw IllegalStateException("Unsupported platform: ${platformName()}")
    }
}
