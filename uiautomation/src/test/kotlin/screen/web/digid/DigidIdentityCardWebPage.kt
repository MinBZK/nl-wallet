package screen.web.digid

import org.openqa.selenium.By
import util.MobileActions

class DigidIdentityCardWebPage : MobileActions() {

    private val headlineTextLocator = By.xpath("//h1[text() = 'Identiteitsbewijs']")

    fun visible() = isWebElementVisible(findElement(headlineTextLocator))
}
