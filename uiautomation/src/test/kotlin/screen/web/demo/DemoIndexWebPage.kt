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
    private val loyaltyButtonLocator = By.xpath("//a[@id='loyalty']")
    private val museumMaandkaartButtonLocator = By.xpath("//a[@id='museum_maandkaart']")


    fun visible() = isWebElementVisible(findWebElement(headerTextLocator))

    fun clickAmsterdamMdocButton() = clickWebElementWithMouseEvent(findWebElement(amsterdamMdocButtonLocator))

    fun clickAmsterdamSdJwtButton() = clickWebElementWithMouseEvent(findWebElement(amsterdamSdJwtButtonLocator))

    fun clickXyzBankMdocButton() = clickWebElementWithMouseEvent(findWebElement(xyzBankMdocButtonLocator))

    fun clickXyzBankSdJwtButton() = clickWebElementWithMouseEvent(findWebElement(xyzBankSdJwtButtonLocator))

    fun clickXyzBankSdJwtEuPidButton() = clickWebElementWithMouseEvent(findWebElement(xyzBankSdJwtEuPidButtonLocator))

    fun clickMarketplaceButton() = clickWebElementWithMouseEvent(findWebElement(marketplaceButtonLocator))

    fun clickMonkeyBikeButton() = clickWebElementWithMouseEvent(findWebElement(monkeyBikeButtonLocator))

    fun clickHollandUniversityMdocButton() {
        scrollToWebElement(hollandUniversityMdocButtonLocator)
        clickWebElementWithMouseEvent(findWebElement(hollandUniversityMdocButtonLocator))
    }

    fun clickHollandUniversitySdJwtButton() {
        scrollToWebElement(hollandUniversitySdJwtButtonLocator)
        clickWebElementWithMouseEvent(findWebElement(hollandUniversitySdJwtButtonLocator))
    }

    fun clickInsuranceButton() {
        scrollToWebElement(insuranceButtonLocator)
        clickWebElementWithMouseEvent(findWebElement(insuranceButtonLocator))
    }

    fun clickJobFinderButton() {
        scrollToWebElement(jobFinderButtonLocator)
        clickWebElementWithMouseEvent(findWebElement(jobFinderButtonLocator))
    }

    fun clickLoyaltyButton() {
        scrollToWebElement(loyaltyButtonLocator)
        clickWebElementWithMouseEvent(findWebElement(loyaltyButtonLocator))
    }

    fun clickMuseumMaandkaartButton() {
        scrollToWebElement(museumMaandkaartButtonLocator)
        clickWebElementWithMouseEvent(findWebElement(museumMaandkaartButtonLocator))
    }
}
