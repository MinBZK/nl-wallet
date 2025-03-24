package screen.card

import util.MobileActions

class CardDataScreen : MobileActions() {

    private val screen = find.byValueKey("cardDataScreen")
    private val dataPrivacyBanner = find.byValueKey("dataPrivacyBanner")

    private val pidFirstNameLabel = find.byText("Voornaam")
    private val pidLastNameLabel = find.byText("Achternaam")
    private val birthDateLabel = find.byText("Geboortedatum")
    private val birthDateValue = find.byText("24 maart 2000")

    private val pidFirstNameLabelEnglish = find.byText("First name")
    private val pidLastNameLabelEnglish = find.byText("Name")
    private val pidBirthDateValueEnglish = find.byText("March 24, 2000")

    private val dataIncorrectButton = find.byText(l10n.getString("cardDataScreenIncorrectCta"))
    private val bottomBackButton = find.byText(l10n.getString("generalBottomBackCta"))

    private val scrollableType = ScrollableType.CustomScrollView

    fun visible() = isElementVisible(screen)

    fun dataPrivacyBannerVisible() = isElementVisible(dataPrivacyBanner)

    fun dataAttributesVisible() = isElementVisible(pidFirstNameLabel) &&
        isElementVisible(pidLastNameLabel) &&
        isElementVisible(birthDateLabel) &&
        isElementVisible(birthDateValue)

    fun englishDataLabelsVisible() = isElementVisible(pidFirstNameLabelEnglish) &&
        isElementVisible(pidLastNameLabelEnglish)

    fun englishDataValuesVisible() = isElementVisible(pidBirthDateValueEnglish)

    fun clickDataIncorrectButton() = clickElement(dataIncorrectButton)

    fun clickBottomBackButton() = clickElement(bottomBackButton)

    fun scrollToEnd() = scrollToEnd(scrollableType)
}
