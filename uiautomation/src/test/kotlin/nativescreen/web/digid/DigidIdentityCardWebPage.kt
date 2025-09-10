package nativescreen.web.digid

import org.openqa.selenium.By
import util.NativeMobileActions

class DigidIdentityCardWebPage : NativeMobileActions() {

    private val headlineTextLocator = By.xpath("//h1[text() = 'Identiteitsbewijs']")

    fun visible() = isWebElementVisible(findElement(headlineTextLocator))
}
