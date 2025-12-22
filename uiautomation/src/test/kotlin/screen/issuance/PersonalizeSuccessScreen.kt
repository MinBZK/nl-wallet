package screen.issuance

import util.MobileActions

class PersonalizeSuccessScreen : MobileActions() {

    private val successTitleText = l10n.getString("walletPersonalizeSuccessPageTitle")
    private val successDescriptionText = l10n.getString("walletPersonalizeSuccessPageDescription")
    private val pidIdCardTitleText = cardMetadata.getPidDisplayName()
    private val nextButton = l10n.getString("walletPersonalizeSuccessPageContinueCta")

    fun visible() = elementWithTextVisible(successTitleText)

    fun successMessageVisible() =
        elementWithTextVisible(successTitleText) && elementWithTextVisible(successDescriptionText)

    fun cardVisible() =
         elementContainingTextVisible(pidIdCardTitleText)

    fun clickNextButton() = clickElementWithText(nextButton)
}
