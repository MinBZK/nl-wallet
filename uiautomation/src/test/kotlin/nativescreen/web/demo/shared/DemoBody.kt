package nativescreen.web.demo.shared

import org.openqa.selenium.By
import util.NativeMobileActions

class DemoBody : NativeMobileActions() {

    private val startButtonLocator = By.xpath("//nl-wallet-button")

    fun clickStartButton() = clickWebElement(findElement(startButtonLocator))
}
