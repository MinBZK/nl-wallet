package screen.introduction

import util.MobileActions

class AppTourScreen : MobileActions() {

    private val backButton = l10n.getString("generalBottomBackCta")
    private val title = l10n.getString("tourOverviewScreenTitle")
    private val subtitle = l10n.getString("tourOverviewScreenSubtitle")
    private val videoTitle = l10n.getString("videoTitle_intro")
    private val videoPlayButton = l10n.getString("tourOverviewScreenItemWCAGLabel").replace("{name}", l10n.getString("videoTitle_intro"))

    fun clickBackButton() {
        scrollToElementWithText(backButton)
        clickElementWithText(backButton)
    }

    fun headlineVisible() = elementWithTextVisible(title)

    fun descriptionVisible() = elementContainingTextVisible(subtitle.substringBefore("'"))

    fun videoTitleVisible() = elementWithTextVisible(videoTitle)

    fun videoPlayButtonVisible() = elementWithTextVisible(videoPlayButton)

    fun playVideo() = clickElementWithText(videoPlayButton)

}
