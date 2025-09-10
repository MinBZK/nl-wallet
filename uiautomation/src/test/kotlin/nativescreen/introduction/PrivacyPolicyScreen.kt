package nativescreen.introduction

import util.NativeMobileActions

class PrivacyPolicyScreen : NativeMobileActions() {

    private val screenTitle = "Privacyverklaring Publieke Wallet"
    private val backButton = l10n.getString("generalBottomBackCta")

    fun visible() = elementContainingTextVisible(screenTitle)

    fun clickBackButton() = clickElementWithText(backButton)
}
