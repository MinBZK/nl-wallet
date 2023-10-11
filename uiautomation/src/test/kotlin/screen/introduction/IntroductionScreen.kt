package screen.introduction

import com.codeborne.selenide.WebDriverRunner
import io.github.ashwith.flutter.FlutterFinder

import io.qameta.allure.Step
import org.openqa.selenium.remote.RemoteWebDriver

import util.MobileActions

class IntroductionScreen : MobileActions() {

    private val find = FlutterFinder(WebDriverRunner.getWebDriver() as RemoteWebDriver)
    private val nextButtonText = find.byValueKey("introductionNextPageCtaText")
    private val nextButton = find.byValueKey("introductionNextPageCta")
    private val skipButton = find.byValueKey("introductionSkipCta")

    @Step("read next button text")
    fun readNextButtonText(): String? {
        return readText(nextButtonText)
    }

    @Step("click next button")
    fun clickNextButton() {
        tapElement(nextButton)
    }

    @Step("click skip button")
    fun clickSkipButton() {
        tapElement(skipButton)
    }
}
