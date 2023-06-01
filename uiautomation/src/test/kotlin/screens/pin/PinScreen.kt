package screens.pin

import com.codeborne.selenide.WebDriverRunner.getWebDriver
import io.github.ashwith.flutter.FlutterFinder
import io.qameta.allure.Step
import org.openqa.selenium.remote.RemoteWebDriver
import util.MobileActions

class PinScreen : MobileActions() {

    private val find = FlutterFinder(getWebDriver() as RemoteWebDriver)
    private val pinScreen = find.byValueKey("select_pin")
    private val simpleErrorMessageTitle =
        find.byValueKey("setupSecurityPinPageSimpleErrorMessageTitle")
    private val simpleErrorMessageDescription =
        find.byValueKey("setupSecurityPinPageSimpleErrorMessageDescription")
    private val differentAccessCodeErrorMessageTitle =
        find.byValueKey("setupSecurityConfirmationErrorPageTitle")
    private val differentAccessCodeErrorMessageDescription =
        find.byValueKey("setupSecurityConfirmationErrorPageDescription")


    @Step("tap the following number: {number}")
    fun tapKeyNumber(number: String) {
        for (digit in number) {
            val elementKey = "keyboardDigitKey#$digit"
            tapElement(find.byValueKey(elementKey))
        }
    }


    @Step("verify if the pin screen is visible")
    fun verifyIfPinScreenIsVisible(): Boolean?{
        return isVisible(pinScreen)
    }

    @Step("verify if simple error description title is visible")
    fun verifySimpleErrorMessageDescriptionText(): String? {
        return verifyText(simpleErrorMessageDescription)
    }

    @Step("verify if simple error message title is visible")
    fun verifySimpleErrorMessageTitleText(): String? {
        return verifyText(simpleErrorMessageTitle)
    }

    @Step("verify if different access code error message title is visible")
    fun verifyErrorTitleDifferentAccessCode(): String? {
        return verifyText(differentAccessCodeErrorMessageTitle)
    }

    @Step("verify if different access code error message description is visible")
    fun verifyErrorDescriptionDifferentAccessCode(): String? {
        return verifyText(differentAccessCodeErrorMessageDescription)
    }
}
