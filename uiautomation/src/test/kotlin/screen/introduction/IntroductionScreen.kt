package screen.introduction

import com.codeborne.selenide.WebDriverRunner
import io.github.ashwith.flutter.FlutterFinder

import io.qameta.allure.Step
import org.openqa.selenium.remote.RemoteWebDriver

import util.MobileActions

class IntroductionScreen : MobileActions() {

    private val find = FlutterFinder(WebDriverRunner.getWebDriver() as RemoteWebDriver)
    private val nextButtonText = find.byValueKey("introductionNextPageCtaText")
    private val privacyPolicyButton = find.byValueKey("introductionPrivacyPolicyCta")
    private val skipButton = find.byValueKey("introductionSkipCta")

    @Step("click next button")
    fun clickNextButton() {
        tapElement(nextButtonText)
    }

    @Step("click privacy policy button")
    fun clickPrivacyPolicyButton() {
        tapElement(privacyPolicyButton)
    }

    @Step("click skip button")
    fun clickSkipButton() {
        tapElement(skipButton)
    }

    @Step("read next button text")
    fun readNextButtonText(): String? {
        return readText(nextButtonText)
    }
}
