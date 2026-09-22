package screen.web.digid

import org.openqa.selenium.By
import org.openqa.selenium.JavascriptExecutor
import util.MobileActions

class DigidLoginMockWebPage : MobileActions() {

    private val listHeadingLocator = By.xpath("//h2[contains(@class, 'list-heading')]")
    private fun cardButtonLocator(bsn: String) = By.xpath(
        "//form[contains(@class, 'card') and .//input[@name='bsn' and @value='$bsn']]//button"
    )

    fun visible(): Boolean {
        return isWebElementVisible(findWebElement(listHeadingLocator))
    }

    fun selectTestId(bsn: String) {
        val button = findWebElement(cardButtonLocator(bsn))
        isWebElementVisible(button)
        clickWebElementWithMouseEvent(button)
    }

    fun login(bsn: String) {
        switchToWebViewWindowContaining(cardButtonLocator(bsn))
        selectTestId(bsn)
    }

    // Old free-form BSN entry is gone from the UI. Rewrite the first card via JS,
    // then submit it as usual so the server-side failure path stays covered.
    fun loginWithFakeBsn(fakeBsn: String = "123456789", fakeName: String = "Niet-Bestaande Gebruiker") {
        switchToWebViewWindowContaining(listHeadingLocator)
        (driver as JavascriptExecutor).executeScript(
            "const form = document.querySelector(\"form.card\");" +
                "form.querySelector(\"input[name='bsn']\").value = arguments[0];" +
                "form.querySelector(\".name\").textContent = arguments[1];" +
                "form.querySelector(\".bsn\").textContent = \"BSN \" + arguments[0];",
            fakeBsn,
            fakeName,
        )
        selectTestId(fakeBsn)
    }
}
