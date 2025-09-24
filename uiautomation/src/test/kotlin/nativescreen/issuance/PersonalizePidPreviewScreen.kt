package nativescreen.issuance

import util.NativeMobileActions

class PersonalizePidPreviewScreen : NativeMobileActions() {

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
