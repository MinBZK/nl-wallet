package screen.pin

import com.codeborne.selenide.WebDriverRunner.getWebDriver
import io.github.ashwith.flutter.FlutterFinder
import io.qameta.allure.Step
import org.openqa.selenium.remote.RemoteWebDriver
import util.MobileActions

class PinScreen : MobileActions() {

    private val find = FlutterFinder(getWebDriver() as RemoteWebDriver)
    private val screen = find.byValueKey("select_pin")
    private val simpleErrorMessageTitle = find.byValueKey("setupSecurityPinPageSimpleErrorMessageTitle")
    private val simpleErrorMessageDescription = find.byValueKey("setupSecurityPinPageSimpleErrorMessageDescription")
    private val differentAccessCodeErrorMessageTitle = find.byValueKey("setupSecurityConfirmationErrorPageTitle")
    private val differentAccessCodeErrorMessageDescription = find.byValueKey("setupSecurityConfirmationErrorPageDescription")

    @Step("wait for pin screen visibility")
    fun waitForScreenVisibility(): Boolean {
        return waitForVisibility(screen)
    }

    @Step("tap the following number: {number}")
    fun clickKeyNumber(number: String) {
        for (digit in number) {
            val elementKey = "keyboardDigitKey#$digit"
            tapElement(find.byValueKey(elementKey))
        }
    }

    @Step("verify if simple error description title is visible")
    fun readSimpleErrorMessageDescriptionText(): String? {
        return readText(simpleErrorMessageDescription)
    }

    @Step("verify if simple error message title is visible")
    fun readSimpleErrorMessageTitleText(): String? {
        return readText(simpleErrorMessageTitle)
    }

    @Step("verify if different access code error message title is visible")
    fun readErrorTitleDifferentAccessCodeText(): String? {
        return readText(differentAccessCodeErrorMessageTitle)
    }

    @Step("verify if different access code error message description is visible")
    fun readErrorDescriptionDifferentAccessCodeText(): String? {
        return readText(differentAccessCodeErrorMessageDescription)
    }
}
