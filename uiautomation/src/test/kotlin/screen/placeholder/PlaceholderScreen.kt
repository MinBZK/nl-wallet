package screen.placeholder

import com.codeborne.selenide.WebDriverRunner.getWebDriver
import io.github.ashwith.flutter.FlutterFinder
import io.qameta.allure.Step
import org.openqa.selenium.remote.RemoteWebDriver
import util.MobileActions

class PlaceholderScreen : MobileActions() {

    private val find = FlutterFinder(getWebDriver() as RemoteWebDriver)
    private val screen = find.byValueKey("placeholderScreenKey")

    @Step("wait for placeholder screen visibility")
    fun waitForScreenVisibility(): Boolean {
        return waitForVisibility(screen)
    }
}
