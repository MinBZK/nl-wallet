package screen.web.demo.issuer

import org.openqa.selenium.By
import org.openqa.selenium.JavascriptExecutor
import util.MobileActions

class IssuerConsentWebPage : MobileActions() {

    private val addToWalletButtonLocator = By.xpath("//section[@class='buttons']//button")

    fun visible() = isWebElementVisible(findWebElement(addToWalletButtonLocator))

    fun clickAddToWalletButton() {
        val element = findWebElement(addToWalletButtonLocator)
        (driver as JavascriptExecutor).executeScript("arguments[0].click()", element)
    }
}
