package screen.introduction

import util.MobileActions

class AppTourScreen : MobileActions() {

    private val backButton = find.byText(l10n.getString("generalBottomBackCta"))
    private val title = find.byText(l10n.getString("tourOverviewScreenTitle"))
    private val subtitle = find.byText(l10n.getString("tourOverviewScreenSubtitle"))
    private val videoTitle = find.byText(l10n.getString("videoTitle_intro"))
    private val videoPlayButton = find.bySemanticsLabel(l10n.getString("tourOverviewScreenItemWCAGLabel").replace("{name}", l10n.getString("videoTitle_intro")))
    private val scrollableElement = find.byType(ScrollableType.CustomScrollView.toString())

    fun clickBackButton() {
        scrollToEnd(scrollableElement)
        clickElement(backButton, false)
    }

    fun headlineVisible() = isElementVisible(title)

    fun descriptionVisible() = isElementVisible(subtitle)

    fun videoTitleVisible() = isElementVisible(videoTitle)

    fun videoPlayButtonVisible() = isElementVisible(videoPlayButton)

    fun playVideo() = clickElement(videoPlayButton)

}
