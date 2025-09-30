package screen.issuance

import util.MobileActions

class PersonalizePidPreviewScreen : MobileActions() {

    private val walletPersonalizeCheckDataOfferingPageTitle = l10n.getString("walletPersonalizeCheckDataOfferingPageTitle")
    private val acceptButton = l10n.getString("walletPersonalizeCheckDataOfferingPageAcceptCta")
    private val rejectButton = l10n.getString("walletPersonalizeCheckDataOfferingPageDeclineCtaSemanticsLabel")

    fun visible() = elementWithTextVisible(walletPersonalizeCheckDataOfferingPageTitle)

    fun humanReadableCardDataVisible(cardData: String) = elementContainingTextVisible(cardData)

    fun confirmButtonsVisible(): Boolean {
        scrollToElementWithText(rejectButton)
        return elementWithTextVisible(acceptButton) && elementWithTextVisible(rejectButton)
    }

    fun clickAcceptButton() {
        switchToNativeContext()
        visible()
        scrollToElementWithText(acceptButton)
        clickElementWithText(acceptButton)
    }

    fun clickRejectButton() {
        scrollToElementWithText(rejectButton)
        clickElementWithText(rejectButton)
    }
}
