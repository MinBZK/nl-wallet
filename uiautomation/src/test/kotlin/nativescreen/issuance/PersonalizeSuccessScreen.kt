package nativescreen.issuance

import util.NativeMobileActions

class PersonalizeSuccessScreen : NativeMobileActions() {

    private val successTitleText = l10n.getString("walletPersonalizeSuccessPageTitle")
    private val successDescriptionText = l10n.getString("walletPersonalizeSuccessPageDescription")
    private val pidIdCardTitleText = cardMetadata.getPidDisplayName()
    private val pidAddressCardTitleText = cardMetadata.getAddressDisplayName()
    private val nextButton = l10n.getString("walletPersonalizeSuccessPageContinueCta")

    fun visible() = elementWithTextVisible(successTitleText)

    fun successMessageVisible() =
        elementWithTextVisible(successTitleText) && elementWithTextVisible(successDescriptionText)

    fun cardsVisible(): Boolean {
        return elementContainingTextVisible(pidIdCardTitleText) && elementContainingTextVisible(pidAddressCardTitleText)
    }

    fun clickNextButton() = clickElementWithText(nextButton)
}
