package screen.web.digid

import org.openqa.selenium.By
import org.openqa.selenium.JavascriptExecutor
import org.openqa.selenium.support.ui.WebDriverWait
import util.MobileActions
import java.time.Duration

class DigidLoginMockWebPage : MobileActions() {

    private val headlineTextLocator = By.xpath("//*[contains(text(), 'DigiD MOCK')]")
    private val mockLoginButtonLocator = By.xpath("//button[normalize-space()='Login / Submit']")

    fun visible(): Boolean {
        return isWebElementVisible(findWebElement(headlineTextLocator))
    }

    fun enterBsn(bsn: String) {
        switchToWebViewContext()
        val js = driver as JavascriptExecutor
        WebDriverWait(driver, Duration.ofSeconds(8)).until {
            js.executeScript("return document.getElementById('bsn_inp') != null") as Boolean
        }
        js.executeScript(
            """
            const el = document.getElementById('bsn_inp');
            el.value = arguments[0];
            el.dispatchEvent(new Event('input', {bubbles:true}));
            el.dispatchEvent(new Event('change', {bubbles:true}));
            """.trimIndent(),
            bsn
        )
    }

    fun clickLoginButton() = findWebElement(mockLoginButtonLocator).click()

    fun login(bsn: String) {
        enterBsn(bsn)
        clickLoginButton()
    }
}
