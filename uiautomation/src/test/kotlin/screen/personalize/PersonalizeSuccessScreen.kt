package screen.personalize

import util.MobileActions

class PersonalizeSuccessScreen : MobileActions() {

    private val screen = find.byValueKey("personalizeSuccessPage")

    private val successTitleText = find.byText(l10n.getString("walletPersonalizeSuccessPageTitle"))
    private val successDescriptionText = find.byText(l10n.getString("walletPersonalizeSuccessPageDescription"))
    private val pidIdCardTitleText = find.byText(cardMetadata.getPidDisplayName())
    private val pidAddressCardTitleText = find.byText(cardMetadata.getAddressDisplayName())

    private val nextButton = find.byValueKey("primaryButtonCta")

    fun visible() = isElementVisible(screen, false)

    fun successMessageVisible() =
        isElementVisible(successTitleText, false) && isElementVisible(successDescriptionText, false)

    fun cardsVisible(): Boolean {
        return isElementVisible(pidIdCardTitleText, false) && isElementVisible(pidAddressCardTitleText, false)
    }

    fun clickNextButton() = clickElement(nextButton, false)
}
