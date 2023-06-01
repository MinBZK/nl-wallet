package screens.pin

import com.codeborne.selenide.WebDriverRunner
import io.github.ashwith.flutter.FlutterFinder
import io.qameta.allure.Step
import org.openqa.selenium.remote.RemoteWebDriver
import util.MobileActions

class SetupSecurityCompletedScreen : MobileActions() {

    private val find = FlutterFinder(WebDriverRunner.getWebDriver() as RemoteWebDriver)
    private val setupSecurityCompletedScreen = find.byValueKey("setupSecurityCompletedPageKey")

    @Step("verify if the setup security completed screen is visible")
    fun verifyIfSetupSecurityCompletedScreenIsVisible(): Boolean? {
        return isVisible(setupSecurityCompletedScreen)
    }
}
