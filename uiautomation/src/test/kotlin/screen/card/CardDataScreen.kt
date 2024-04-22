package screen.card

import util.MobileActions

class CardDataScreen : MobileActions() {

    private val screen = find.byValueKey("cardDataScreen")
    private val dataPrivacyBanner = find.byValueKey("dataPrivacyBanner")

    private val pidFirstNamesLabel = find.byText("Voornamen")
    private val pidFirstNamesValue = find.byText("Frouke")
    private val pidLastNameLabel = find.byText("Achternaam")
    private val pidLastNameValue = find.byText("Jansen")
    private val birthDateLabel = find.byText("Geboortedatum")
    private val birthDateValue = find.byText("24 maart 2000")

    private val pidFirstNamesLabelEnglish = find.byText("First names")
    private val pidLastNameLabelEnglish = find.byText("Surname")
    private val pidBirthDateValueEnglish = find.byText("March 24, 2000")

    private val dataPrivacySheetTitle = find.byText(l10n.getString("cardDataScreenDataPrivacySheetTitle"))
    private val dataPrivacySheetDescription = find.byText(l10n.getString("cardDataScreenDataPrivacySheetDescription"))

    private val dataIncorrectButton = find.byText(l10n.getString("cardDataScreenIncorrectCta"))
    private val bottomBackButton = find.byText(l10n.getString("generalBottomBackCta"))

    private val scrollableType = ScrollableType.CustomScrollView

    fun visible() = isElementVisible(screen)

    fun dataPrivacyBannerVisible() = isElementVisible(dataPrivacyBanner)

    fun dataAttributesVisible() = isElementVisible(pidFirstNamesLabel) &&
        isElementVisible(pidLastNameLabel) &&
        isElementVisible(pidFirstNamesValue) &&
        isElementVisible(pidLastNameValue) &&
        isElementVisible(birthDateLabel) &&
        isElementVisible(birthDateValue)

    fun englishDataLabelsVisible() = isElementVisible(pidFirstNamesLabelEnglish) &&
        isElementVisible(pidLastNameLabelEnglish)

    fun englishDataValuesVisible() = isElementVisible(pidBirthDateValueEnglish)

    fun dataPrivacySheetVisible() = isElementVisible(dataPrivacySheetTitle) &&
        isElementVisible(dataPrivacySheetDescription)

    fun clickDataPrivacyBanner() = clickElement(dataPrivacyBanner)

    fun clickDataIncorrectButton() = clickElement(dataIncorrectButton)

    fun clickBottomBackButton() = clickElement(bottomBackButton)

    fun scrollToEnd() = scrollToEnd(scrollableType)
}
