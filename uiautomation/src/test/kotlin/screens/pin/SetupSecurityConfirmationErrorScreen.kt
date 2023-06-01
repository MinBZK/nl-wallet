package screens.pin

import com.codeborne.selenide.WebDriverRunner
import io.github.ashwith.flutter.FlutterFinder
import io.qameta.allure.Step
import org.openqa.selenium.remote.RemoteWebDriver
import util.MobileActions

class SetupSecurityConfirmationErrorScreen : MobileActions() {

    private val find = FlutterFinder(WebDriverRunner.getWebDriver() as RemoteWebDriver)
    private val fatalErrorTitle =
        find.byValueKey("setupSecurityConfirmationErrorPageFatalTitle")
    private val fatalErrorDescription =
        find.byValueKey("setupSecurityConfirmationErrorPageFatalDescription")
    private val selectNewCode =
        find.byValueKey("setupSecurityConfirmationErrorPageFatalCta")

    @Step("click select new code button")
    fun clickSelectNewCodeButton() {
        tapElement(selectNewCode)
    }

    @Step("verify if fatal error title is visible")
    fun verifyErrorConfirmationFatalErrorTitle(): String? {
        return verifyText(fatalErrorTitle)
    }

    @Step("verify if fatal error description is visible")
    fun verifyErrorConfirmationFatalErrorDescription(): String? {
        return verifyText(fatalErrorDescription)
    }
}
