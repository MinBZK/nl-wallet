package screens.introduction

import com.codeborne.selenide.Selenide

import com.codeborne.selenide.WebDriverRunner.getWebDriver
import io.github.ashwith.flutter.FlutterFinder
import io.qameta.allure.Step
import org.openqa.selenium.remote.RemoteWebDriver

object IntroductionPrivacyPolicyScreen {
    //TODO: text input should be changed
    private val find = FlutterFinder(getWebDriver() as RemoteWebDriver)
    private val placeholderText = Selenide.`$`(find.byText("Helaas, aan deze pagina wordt nog gewerkt."))

    @Step("verify if the placeholder text is visible in the Privacy Policy screen")
    fun verifyPlaceholderTextIsVisible(): Boolean {
        return placeholderText.exists()
    }
}