package screen.pin

import com.codeborne.selenide.WebDriverRunner
import io.github.ashwith.flutter.FlutterFinder
import io.qameta.allure.Step
import org.openqa.selenium.remote.RemoteWebDriver
import util.MobileActions

class SetupSecurityCompletedScreen : MobileActions() {

    private val find = FlutterFinder(WebDriverRunner.getWebDriver() as RemoteWebDriver)
    private val screen = find.byValueKey("setupSecurityCompletedPageKey")

    @Step("wait for setup security completed screen visibility")
    fun waitForScreenVisibility(): Boolean {
        return waitForVisibility(screen)
    }
}
