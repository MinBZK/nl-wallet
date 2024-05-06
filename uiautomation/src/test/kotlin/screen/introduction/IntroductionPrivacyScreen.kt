package screen.introduction

import util.MobileActions

class IntroductionPrivacyScreen : MobileActions() {

    private val screen = find.byValueKey("introductionPrivacyScreen")

    private val privacyButton = find.byValueKey("introductionPrivacyScreenPrivacyCta")
    private val nextButton = find.byValueKey("introductionPrivacyScreenNextCta")
    private val backButton = find.byToolTip(l10n.getString("generalWCAGBack"))

    fun visible() = isElementVisible(screen)

    fun absent() = isElementAbsent(screen, false)

    fun clickPrivacyButton() = clickElement(privacyButton)

    fun clickNextButton() = clickElement(nextButton)

    fun clickBackButton() = clickElement(backButton)
}
