package screen.introduction

import util.MobileActions

class IntroductionExpectationsScreen : MobileActions() {

    private val screen = find.byValueKey("introductionExpectationsScreen")

    private val nextButton = find.byValueKey("introductionExpectationsScreenCta")
    private val backButton = find.byValueKey("introductionBackCta")

    fun visible() = isElementVisible(screen)

    fun clickNextButton() = clickElement(nextButton)

    fun clickBackButton() = clickElement(backButton)
}
