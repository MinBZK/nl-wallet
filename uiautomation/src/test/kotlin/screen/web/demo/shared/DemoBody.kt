package screen.web.demo.shared

import org.openqa.selenium.By
import util.MobileActions

class DemoBody : MobileActions() {

    private val startButtonLocator = By.xpath("//nl-wallet-button")

    fun clickStartButton() = clickWebElement(findElement(startButtonLocator))
}
