package screen.introduction

import util.MobileActions

class IntroductionScreen : MobileActions() {

    private val page1 = find.byValueKey("introductionPage1")
    private val page2 = find.byValueKey("introductionPage2")
    private val page3 = find.byValueKey("introductionPage3")

    private val nextButton = find.byValueKey("introductionNextPageCta")
    private val skipButton = find.byValueKey("introductionSkipCta")
    private val backButton = find.byToolTip(l10n.getString("generalWCAGBack"))

    fun page1Visible() = isElementVisible(page1)

    fun page2Visible() = isElementVisible(page2)

    fun page3Visible() = isElementVisible(page3)

    fun nextButtonTextVisible(text: String) = isElementVisible(find.byText(text))

    fun clickNextButton() = clickElement(nextButton)

    fun clickSkipButton() = clickElement(skipButton)

    fun clickBackButton() = clickElement(backButton)
}
