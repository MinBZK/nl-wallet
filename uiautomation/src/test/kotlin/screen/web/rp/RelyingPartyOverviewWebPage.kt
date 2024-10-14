package screen.web.rp

import org.openqa.selenium.By
import util.MobileActions

class RelyingPartyOverviewWebPage : MobileActions() {

    private val headerTextLocator = By.xpath("//h1[text()='NL Wallet demo']")

    private val amsterdamButtonLocator = By.xpath("//a[@id='mijn_amsterdam']")
    private val xyzBankButtonLocator = By.xpath("//a[@id='xyz_bank']")
    private val marketplaceButtonLocator = By.xpath("//a[@id='online_marketplace']")
    private val monkeyBikeButtonLocator = By.xpath("//a[@id='monkey_bike']")

    fun visible() = isWebElementVisible(findElement(headerTextLocator))

    fun clickAmsterdamButton() = clickWebElement(findElement(amsterdamButtonLocator))

    fun clickXyzBankButton() = clickWebElement(findElement(xyzBankButtonLocator))

    fun clickMarketplaceButton() = clickWebElement(findElement(marketplaceButtonLocator))

    fun clickMonkeyBikeButton() = clickWebElement(findElement(monkeyBikeButtonLocator))
}
