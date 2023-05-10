package screens.introduction

import com.codeborne.selenide.WebDriverRunner
import io.github.ashwith.flutter.FlutterFinder

import io.qameta.allure.Step
import org.openqa.selenium.remote.RemoteWebDriver

import util.MobileActions

class IntroductionChangeLanguageScreen {

    private val find = FlutterFinder(WebDriverRunner.getWebDriver() as RemoteWebDriver)
    private val changeLanguageScreenBackButton = find.byValueKey("changeLanguageScreenBackCta")
    private val selectEnglishLanguage = find.byText("English")
    private val selectDutchLanguage = find.byText("Nederlands")

    @Step("click back button")
    fun clickBackButton() {
        changeLanguageScreenBackButton.click()
    }

    @Step("select english language option")
    fun selectEnglishLanguageOption() {
        selectEnglishLanguage.click()
    }

    @Step("select dutch language option")
    fun selectDutchLanguageOption() {
        selectDutchLanguage.click()
    }

}