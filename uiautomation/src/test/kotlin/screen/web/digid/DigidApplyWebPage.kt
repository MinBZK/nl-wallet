package screen.web.digid

import org.openqa.selenium.By
import util.MobileActions

class DigidApplyWebPage : MobileActions() {

    private val headlineTextLocator = By.xpath("//h1[contains(text(), 'DigiD aanvragen')]")

    fun visible() = isWebElementVisible(findElement(headlineTextLocator))
}
