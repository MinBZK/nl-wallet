package screens.introduction

import com.codeborne.selenide.WebDriverRunner.getWebDriver
import io.github.ashwith.flutter.FlutterFinder
import io.qameta.allure.Step
import org.openqa.selenium.remote.RemoteWebDriver
import util.MobileActions

class IntroductionPrivacyPolicyScreen : MobileActions() {

    private val find = FlutterFinder(getWebDriver() as RemoteWebDriver)
    private val placeholderScreen = find.byValueKey("placeholderScreenKey")

    @Step("verify if the placeholderScreen is visible in the Privacy Policy screen")
    fun verifyPlaceholderScreenIsVisible(): Boolean? {
        return isVisible(placeholderScreen)
    }
}
