package screen.introduction

import util.MobileActions

class IntroductionScreen : MobileActions() {

    private val introductionPage1Title = l10n.getString("introductionPage1Title")
    private val introductionPage2Title = l10n.getString("introductionPage2Title")
    private val introductionPage3Title = l10n.getString("introductionPage3Title")
    private val nextButton = l10n.getString("introductionNextPageCta")
    private val skipButton = l10n.getString("introductionSkipCta")
    private val backButton = l10n.getString("generalWCAGBack")

    fun page1Visible(): Boolean {
        explicitWait()
        return elementWithTextVisible(introductionPage1Title)
    }

    fun page2Visible(): Boolean {
        explicitWait()
        return elementWithTextVisible(introductionPage2Title)
    }

    fun page3Visible(): Boolean {
        explicitWait()
        return elementWithTextVisible(introductionPage3Title)
    }

    fun nextButtonTextVisible(text: String) = elementWithTextVisible(text)

    fun clickNextButton() = clickElementWithText(nextButton)

    fun clickSkipButton() = clickElementWithText(skipButton)

    fun clickBackButton() = clickElementWithText(backButton)

    // Explicit wait/sleep to wait for page transition to finish (and button animations to settle).
    // This is needed due to the animations on the intro pages combined with `frameSync = false`.
    private fun explicitWait() {
        Thread.sleep(1000)
    }
}
