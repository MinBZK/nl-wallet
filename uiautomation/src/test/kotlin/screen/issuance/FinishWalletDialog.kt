package screen.issuance

import util.MobileActions

class FinishWalletDialog : MobileActions() {

    private val finishWalletDialogTitle = l10n.getString("finishSetupDialogDescription")
    private val okButton = l10n.getString("generalOkCta")

    fun visible() = elementWithTextVisible(finishWalletDialogTitle)

    fun clickOkButton() = clickElementWithText(okButton)
}
