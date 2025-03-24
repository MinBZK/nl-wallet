package screen.introduction

import util.MobileActions

class IntroductionScreen : MobileActions() {

    private val page1 = find.byValueKey("introductionPage1")
    private val page2 = find.byValueKey("introductionPage2")
    private val page3 = find.byValueKey("introductionPage3")

    private val nextButton by lazy { find.byText(l10n.getString("introductionNextPageCta")) }
    private val skipButton by lazy { find.byText(l10n.getString("introductionSkipCta")) }
    private val backButton by lazy { find.byToolTip(l10n.getString("generalWCAGBack")) }

    fun page1Visible(): Boolean {
        explicitWait()
        return isElementVisible(page1, false)
    }

    fun page2Visible(): Boolean {
        explicitWait()
        return isElementVisible(page2, false)
    }

    fun page3Visible(): Boolean {
        explicitWait()
        return isElementVisible(page3, false)
    }

    fun nextButtonTextVisible(text: String) = isElementVisible(find.byText(text), false)

    fun clickNextButton() = clickElement(nextButton, false)

    fun clickSkipButton() = clickElement(skipButton, false)

    fun clickBackButton() = clickElement(backButton, false)

    // Explicit wait/sleep to wait for page transition to finish (and button animations to settle).
    // This is needed due to the animations on the intro pages combined with `frameSync = false`.
    private fun explicitWait() {
        Thread.sleep(1000)
    }
}
