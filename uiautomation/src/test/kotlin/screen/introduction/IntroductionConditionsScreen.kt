package screen.introduction

import util.MobileActions

class IntroductionConditionsScreen : MobileActions() {

    private val screen = find.byValueKey("introductionConditionsScreen")

    private val conditionsButton = find.byValueKey("introductionConditionsScreenConditionsCta")
    private val nextButton = find.byValueKey("introductionConditionsScreenNextCta")
    private val backButton = find.byToolTip(l10n.getString("generalWCAGBack"))

    fun visible() = isElementVisible(screen)

    fun absent() = isElementAbsent(screen)

    fun clickConditionsButton() = clickElement(conditionsButton)

    fun clickNextButton() = clickElement(nextButton)

    fun clickBackButton() = clickElement(backButton)
}
