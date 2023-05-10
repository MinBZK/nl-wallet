package screens.introduction

import com.codeborne.selenide.WebDriverRunner
import io.github.ashwith.flutter.FlutterFinder

import io.qameta.allure.Step
import org.openqa.selenium.remote.RemoteWebDriver

import util.MobileActions

class IntroductionScreens : MobileActions(){

    private val find = FlutterFinder(WebDriverRunner.getWebDriver() as RemoteWebDriver)
    private val nextButton = find.byValueKey("introductionNextPageCta")
    private val changeLanguageButton = find.byValueKey("introductionLanguageSelectCta")
    private val changeLanguageButtonText = find.byValueKey("introductionLanguageSelectCtaText")
    private val privacyPolicyButton = find.byValueKey("introductionPrivacyPolicyCta")
    private val backButton = find.byValueKey("introductionBackCta")

    @Step("click next button")
    fun clickNextButton() {
        nextButton.click()
    }

    @Step("click change language button")
    fun clickChangeLanguageButton() {
        changeLanguageButton.click()
    }

    @Step("verify if the privacy policy button is visible")
    fun verifyPrivacyPolicyButtonIsVisible(): Boolean {
        return privacyPolicyButton.isDisplayed
    }

    @Step("click privacy policy button")
    fun clickPrivacyPolicyButton() {
        privacyPolicyButton.click()
    }

    @Step("verify if selected language")
    fun verifySelectedLanguage() : String? {
        return changeLanguageButtonText.text
    }
}