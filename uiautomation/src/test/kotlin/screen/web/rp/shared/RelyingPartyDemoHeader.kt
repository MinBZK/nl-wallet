package screen.web.rp.shared

import org.openqa.selenium.By
import util.MobileActions

class RelyingPartyDemoHeader : MobileActions() {

    private val languageToggleButtonLocator = By.xpath("//label[@for='lang_toggle']")
    private val dutchLanguageButtonLocator = By.xpath("//button[text()='Nederlands']")
    private val englishLanguageButtonLocator = By.xpath("//button[text()='English']")

    fun clickDutchLanguageButton() {
        clickWebElement(findElement(languageToggleButtonLocator))
        clickWebElement(findElement(dutchLanguageButtonLocator))
    }

    fun clickEnglishLanguageButton() {
        clickWebElement(findElement(languageToggleButtonLocator))
        clickWebElement(findElement(englishLanguageButtonLocator))
    }
}
