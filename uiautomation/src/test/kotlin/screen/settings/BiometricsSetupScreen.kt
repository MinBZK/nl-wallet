package screen.settings

import domain.Platform
import io.appium.java_client.AppiumBy
import org.openqa.selenium.By
import util.MobileActions

class BiometricsSetupScreen : MobileActions() {

    private val screenTitle = l10n.getString("biometricSettingsScreenTitle").replace("{supportedBiometrics}", "")
    private val backButton = l10n.getString("generalBottomBackCta")
    private val closeButton = l10n.getString("generalDialogCloseCta")

    fun visible() = elementContainingTextVisible(screenTitle)

    fun toggleBiometricUnlock() {
        when (platform()) {
            Platform.IOS -> driver.findElement(By.className("XCUIElementTypeSwitch")).click()
            Platform.ANDROID -> driver.findElement(AppiumBy.className("android.widget.Switch")).click()
        }
        Thread.sleep(SET_FRAME_SYNC_MAX_WAIT_MILLIS)
        performBiometricAuthentication(true)
        Thread.sleep(SET_FRAME_SYNC_MAX_WAIT_MILLIS)
    }

    fun clickCloseButton() {
        visible()
        Thread.sleep(SET_FRAME_SYNC_MAX_WAIT_MILLIS)
        clickElementWithText(closeButton)
    }

    fun clickBackButton() = clickElementWithText(backButton)
}
