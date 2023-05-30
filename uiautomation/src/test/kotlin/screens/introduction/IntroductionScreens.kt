package screens.introduction

import com.codeborne.selenide.WebDriverRunner
import io.github.ashwith.flutter.FlutterFinder

import io.qameta.allure.Step
import org.openqa.selenium.remote.RemoteWebDriver

import util.MobileActions

class IntroductionScreens : MobileActions() {

    private val find = FlutterFinder(WebDriverRunner.getWebDriver() as RemoteWebDriver)
    private val nextButtonText = find.byValueKey("introductionNextPageCtaText")
    private val privacyPolicyButton = find.byValueKey("introductionPrivacyPolicyCta")
    private val backButton = find.byValueKey("introductionBackCta")

    @Step("click next button")
    fun clickNextButton() {
        tapElement(nextButtonText)
    }

    @Step("click privacy policy button")
    fun clickPrivacyPolicyButton() {
        tapElement(privacyPolicyButton)
    }

    @Step("verify if next button text")
    fun verifyNextButtonText(): String? {
        return nextButtonText.text
    }
}
