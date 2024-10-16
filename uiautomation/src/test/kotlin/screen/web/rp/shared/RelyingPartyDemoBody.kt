package screen.web.rp.shared

import org.openqa.selenium.By
import util.MobileActions

class RelyingPartyDemoBody : MobileActions() {

    private val startButtonLocator = By.xpath("//nl-wallet-button")

    fun clickStartButton() = clickWebElement(findElement(startButtonLocator))

    fun clickBackButton() = navigateBack()
}
