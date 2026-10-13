package screen.help

import helper.LocalizationHelper.Translation.HELP_TOPIC_CARD_ACTIVITY_NOT_RECOGNISED
import util.MobileActions

class ActivitiesHelpScreen : MobileActions() {

    private val title = l10n.getString("menuScreenTourCta")
    private val somethingElseButton = l10n.getString("helpTopicScreenSomethingElseCta")
    private val cardActivitiesButton = l10n.getString("cardHistoryScreenTitle")
    private val bottomBackButton = l10n.getString("generalBottomBackCta")
    private val firstCardActivitiesTopic = l10n.translate(HELP_TOPIC_CARD_ACTIVITY_NOT_RECOGNISED)

    fun visible() = elementContainingTextVisible(title)

    fun clickCardActivitiesButton() = clickElementContainingText(cardActivitiesButton)

    fun clickBottomBackButton() = clickElementWithText(bottomBackButton)


    fun clickSomethingElseButton() {
        scrollToElementWithText(somethingElseButton)
        clickElementWithText(somethingElseButton)
    }

    fun clickFirstCardActivitiesTopicButton() = clickElementContainingText(firstCardActivitiesTopic)
}
