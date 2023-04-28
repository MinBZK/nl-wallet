package screens.introduction


import com.codeborne.selenide.Selenide
import com.codeborne.selenide.WebDriverRunner.getWebDriver
import io.appium.java_client.AppiumBy
import io.github.ashwith.flutter.FlutterFinder
import io.qameta.allure.Step
import org.openqa.selenium.remote.RemoteWebDriver

object IntroductionScreens {
    private val find = FlutterFinder(getWebDriver() as RemoteWebDriver)
    private val nextButton = Selenide.`$`(find.byValueKey("introductionNextPageCta"))

    //TODO: accessibilityId should be changed
    private val welcomeText = Selenide.`$`(AppiumBy.accessibilityId("Welcome to the NL Wallet Demo"))
    private val changeLanguageButton = Selenide.`$`(find.byValueKey("English"))
    private val privacyPolicyButton = Selenide.`$`(find.byValueKey("introductionPrivacyPolicyCta"))

    @Step("click next button")
    fun clickNextButton() {
        nextButton.click()
    }

    @Step("click change language button")
    fun clickChangeLanguageButton() {
        changeLanguageButton.click()
    }

    @Step("click privacy policy button")
    fun clickPrivacyPolicyButton() {
        privacyPolicyButton.click()
    }

    @Step("verify welcome text")
    fun verifyWelcomeTextIsVisible(): Boolean {
        return welcomeText.isDisplayed
    }
}