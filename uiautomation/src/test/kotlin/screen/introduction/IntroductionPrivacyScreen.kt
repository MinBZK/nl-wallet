package screen.introduction

import util.MobileActions

class IntroductionPrivacyScreen : MobileActions() {

    private val introductionPrivacyScreenHeadline = l10n.getString("introductionPrivacyScreenHeadline")
    private val privacyButton = l10n.getString("introductionPrivacyScreenPrivacyCta")
    private val nextButton = l10n.getString("introductionPrivacyScreenNextCta")
    private val backButton = l10n.getString("generalWCAGBack")

    fun visible() = elementWithTextVisible(introductionPrivacyScreenHeadline)

    fun absent() = !elementWithTextVisible(introductionPrivacyScreenHeadline)

    fun clickPrivacyButton() = clickElementWithText(privacyButton)

    fun clickNextButton() = clickElementWithText(nextButton)

    fun clickBackButton() = clickElementWithText(backButton)
}
