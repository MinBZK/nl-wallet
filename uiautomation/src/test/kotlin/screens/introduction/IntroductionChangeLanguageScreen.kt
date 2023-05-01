package screens.introduction

import com.codeborne.selenide.Selenide
import io.appium.java_client.AppiumBy
import io.qameta.allure.Step

object IntroductionChangeLanguageScreen {
    private val changeLanguageScreenBackButton = Selenide.`$`(AppiumBy.accessibilityId("changeLanguageScreenBackCta"))

    @Step("click back button")
    fun clickBackButton() {
        changeLanguageScreenBackButton.click()
    }

    @Step("select english language option")
    fun selectEnglishLanguageOption() {
        changeLanguageScreenBackButton.click()
    }
}
