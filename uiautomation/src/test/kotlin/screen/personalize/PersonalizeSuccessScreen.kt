package screen.personalize

import util.MobileActions

class PersonalizeSuccessScreen : MobileActions() {

    private val screen = find.byValueKey("personalizeSuccessPage")

    private val successTitleText = find.byText(l10n.getString("walletPersonalizeSuccessPageTitle"))
    private val successDescriptionText = find.byText(l10n.getString("walletPersonalizeSuccessPageDescription"))
    private val pidIdCardSubtitleText = find.byText("Jansen")
    private val pidAddressCardTitleText = find.byText(l10n.getString("pidAddressCardTitle"))

    private val nextButton = find.byValueKey("primaryButtonCta")

    fun visible() = isElementVisible(screen, false)

    fun successMessageVisible() =
        isElementVisible(successTitleText, false) && isElementVisible(successDescriptionText, false)

    fun cardsVisible(): Boolean {
        return isElementVisible(pidIdCardSubtitleText, false) && isElementVisible(pidAddressCardTitleText, false)
    }

    fun clickNextButton() = clickElement(nextButton, false)
}
