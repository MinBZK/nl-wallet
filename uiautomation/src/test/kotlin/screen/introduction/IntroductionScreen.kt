package screen.introduction

import util.MobileActions

class IntroductionScreen : MobileActions() {

    private val page1 = find.byValueKey("introductionPage1")
    private val page2 = find.byValueKey("introductionPage2")
    private val page3 = find.byValueKey("introductionPage3")
    private val page4 = find.byValueKey("introductionPage4")

    private val nextButton = find.byValueKey("introductionNextPageCta")
    private val skipButton = find.byValueKey("introductionSkipCta")
    private val backButton = find.byValueKey("introductionBackCta")

    private val nextButtonText = find.byValueKey("introductionNextPageCtaText")

    fun page1Visible() = isElementVisible(page1)

    fun page2Visible() = isElementVisible(page2)

    fun page3Visible() = isElementVisible(page3)

    fun page4Visible() = isElementVisible(page4)

    fun page4Absent() = isElementAbsent(page4)

    fun readNextButtonText() = readElementText(nextButtonText)

    fun clickNextButton() = clickElement(nextButton)

    fun clickSkipButton() = clickElement(skipButton)

    fun clickBackButton() = clickElement(backButton)
}
