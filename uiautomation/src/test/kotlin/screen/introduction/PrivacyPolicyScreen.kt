package screen.introduction

import util.MobileActions

class PrivacyPolicyScreen : MobileActions() {

    private val screenTitle = "Privacyverklaring Publieke Wallet"
    private val backButton = l10n.getString("generalBottomBackCta")

    fun visible() = elementContainingTextVisible(screenTitle)

    fun clickBackButton() = clickElementWithText(backButton)
}
