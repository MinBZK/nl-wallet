package screen.permissions

import domain.Platform
import org.openqa.selenium.By
import org.openqa.selenium.TimeoutException
import org.openqa.selenium.support.ui.ExpectedConditions
import org.openqa.selenium.support.ui.WebDriverWait
import util.MobileActions
import java.time.Duration

class NativePermissionDialog : MobileActions() {

    private val androidDenyButtonId = "com.android.permissioncontroller:id/permission_deny_button"
    private val androidDenyDontAskAgainButtonId = "com.android.permissioncontroller:id/permission_deny_and_dont_ask_again_button"
    private val androidAllowOneTimeButtonId = "com.android.permissioncontroller:id/permission_allow_one_time_button"

    private val iosAlertTimeoutMillis = 5000L

    private fun waitForIosAlert(): Boolean = try {
        WebDriverWait(driver, Duration.ofMillis(iosAlertTimeoutMillis))
            .until(ExpectedConditions.alertIsPresent())
        true
    } catch (e : TimeoutException) {
        throw e
    }

    fun visible(): Boolean = when (platform()) {
        Platform.ANDROID -> driver.findElements(By.id(androidDenyButtonId)).isNotEmpty()
        Platform.IOS -> waitForIosAlert()
    }

    fun deny() = when (platform()) {
        Platform.ANDROID -> clickWebElementWithGesture(findWebElement(By.id(androidDenyButtonId)))
        Platform.IOS -> {
            waitForIosAlert()
            driver.switchTo().alert().dismiss()
        }
    }

    fun denyDontAskAgain() = when (platform()) {
        Platform.ANDROID -> clickWebElementWithGesture(findWebElement(By.id(androidDenyDontAskAgainButtonId)))
        Platform.IOS -> throw UnsupportedOperationException("denyDontAskAgain is not supported on iOS")
    }

    fun allowOneTimeOnly() = when (platform()) {
        Platform.ANDROID -> clickWebElementWithGesture(findWebElement(By.id(androidAllowOneTimeButtonId)))
        Platform.IOS -> throw UnsupportedOperationException("allowOneTimeOnly is not supported on iOS")
    }
}
