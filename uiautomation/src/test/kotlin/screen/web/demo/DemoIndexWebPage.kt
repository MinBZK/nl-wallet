package screen.web.demo

import org.openqa.selenium.By
import util.MobileActions

class DemoIndexWebPage : MobileActions() {

    private val headerTextLocator = By.xpath("//h1[text()='NL Wallet demo']")

    private val amsterdamMdocButtonLocator = By.xpath("//a[@id='mijn_amsterdam_mdoc']")
    private val amsterdamSdJwtButtonLocator = By.xpath("//a[@id='mijn_amsterdam_sd_jwt']")
    private val xyzBankMdocButtonLocator = By.xpath("//a[@id='xyz_bank_mdoc']")
    private val xyzBankSdJwtButtonLocator = By.xpath("//a[@id='xyz_bank_sd_jwt']")
    private val xyzBankSdJwtEuPidButtonLocator = By.xpath("//a[@id='xyz_bank_sd_jwt_eu']")

    private val marketplaceButtonLocator = By.xpath("//a[@id='online_marketplace']")
    private val monkeyBikeButtonLocator = By.xpath("//a[@id='monkey_bike']")
    private val hollandUniversityMdocButtonLocator = By.xpath("//a[@id='university_mdoc']")
    private val hollandUniversitySdJwtButtonLocator = By.xpath("//a[@id='university_sd_jwt']")

    private val insuranceButtonLocator = By.xpath("//a[@id='insurance']")
    private val jobFinderButtonLocator = By.xpath("//a[@id='job_finder']")


    fun visible() = isWebElementVisible(findWebElement(headerTextLocator))

    fun clickAmsterdamMdocButton() = clickWebElement(findWebElement(amsterdamMdocButtonLocator))

    fun clickAmsterdamSdJwtButton() = clickWebElement(findWebElement(amsterdamSdJwtButtonLocator))

    fun clickXyzBankMdocButton() = clickWebElement(findWebElement(xyzBankMdocButtonLocator))

    fun clickXyzBankSdJwtButton() = clickWebElement(findWebElement(xyzBankSdJwtButtonLocator))

    fun clickXyzBankSdJwtEuPidButton() = clickWebElement(findWebElement(xyzBankSdJwtEuPidButtonLocator))

    fun clickMarketplaceButton() = clickWebElement(findWebElement(marketplaceButtonLocator))

    fun clickMonkeyBikeButton() = clickWebElement(findWebElement(monkeyBikeButtonLocator))

    fun clickHollandUniversityMdocButton() {
        scrollToWebElement(hollandUniversityMdocButtonLocator)
        clickWebElement(findWebElement(hollandUniversityMdocButtonLocator))
    }

    fun clickHollandUniversitySdJwtButton() {
        scrollToWebElement(hollandUniversitySdJwtButtonLocator)
        clickWebElement(findWebElement(hollandUniversitySdJwtButtonLocator))
    }

    fun clickInsuranceButton() {
        scrollToWebElement(insuranceButtonLocator)
        clickWebElement(findWebElement(insuranceButtonLocator))
    }

    fun clickJobFinderButton() {
        scrollToWebElement(jobFinderButtonLocator)
        clickWebElement(findWebElement(jobFinderButtonLocator))
    }
}
