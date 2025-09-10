package nativescreen.web.digid

import org.openqa.selenium.By
import org.openqa.selenium.JavascriptExecutor
import util.NativeMobileActions

class DigidLoginMockWebPage : NativeMobileActions() {

    private val headlineTextLocator = By.xpath("//*[contains(text(), 'DigiD MOCK')]")
    private val bsnInputLocator = By.id("bsn_inp")
    private val mockLoginButtonLocator = By.linkText("Login / Submit")

    fun visible(): Boolean {
        return isWebElementVisible(findElement(headlineTextLocator))
    }

    fun enterBsn(bsn: String) {
        val bsnInput = findElement(bsnInputLocator)
        bsnInput.clear()
        bsnInput.sendKeys(bsn)
        (driver as JavascriptExecutor).executeScript(
            """
            const el = arguments[0];
            el.dispatchEvent(new Event('input', {bubbles:true}));
            el.dispatchEvent(new Event('change', {bubbles:true}));
            """.trimIndent(),
            bsnInput
        )
    }

    fun clickLoginButton() = findElement(mockLoginButtonLocator).click()

    fun login(bsn: String) {
        enterBsn(bsn)
        clickLoginButton()
    }
}
