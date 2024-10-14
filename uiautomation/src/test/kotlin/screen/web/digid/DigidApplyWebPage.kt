package screen.web.digid

import org.openqa.selenium.By
import util.MobileActions

class DigidApplyWebPage : MobileActions() {

    private val headlineText = findElement(By.xpath("//h1[text()='DigiD aanvragen']"))

    fun visible() = isWebElementVisible(headlineText)
}
