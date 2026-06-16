package screen.permissions

import domain.Platform
import org.openqa.selenium.By
import org.openqa.selenium.NoAlertPresentException
import util.MobileActions

class NativePermissionDialog : MobileActions() {

    private val androidDenyButtonId = "com.android.permissioncontroller:id/permission_deny_button"
    private val androidDenyDontAskAgainButtonId = "com.android.permissioncontroller:id/permission_deny_and_dont_ask_again_button"
    private val androidAllowOneTimeButtonId = "com.android.permissioncontroller:id/permission_allow_one_time_button"

    fun visible(): Boolean = when (platform()) {
        Platform.ANDROID -> driver.findElements(By.id(androidDenyButtonId)).isNotEmpty()
        Platform.IOS -> try { driver.switchTo().alert(); true } catch (_: NoAlertPresentException) { false }
    }

    fun deny() = when (platform()) {
        Platform.ANDROID -> clickWebElement(findWebElement(By.id(androidDenyButtonId)))
        Platform.IOS -> driver.switchTo().alert().dismiss()
    }

    fun denyDontAskAgain() = when (platform()) {
        Platform.ANDROID -> clickWebElement(findWebElement(By.id(androidDenyDontAskAgainButtonId)))
        Platform.IOS -> throw UnsupportedOperationException("denyDontAskAgain is not supported on iOS")
    }

    fun allowOneTimeOnly() = when (platform()) {
        Platform.ANDROID -> clickWebElement(findWebElement(By.id(androidAllowOneTimeButtonId)))
        Platform.IOS -> throw UnsupportedOperationException("allowOneTimeOnly is not supported on iOS")
    }
}
