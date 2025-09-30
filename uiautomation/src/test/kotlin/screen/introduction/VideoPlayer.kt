package screen.introduction

import util.MobileActions

class VideoPlayer : MobileActions() {

    private val closeButton = l10n.getString("videoPlayerCloseTooltip")
    private val subtitlesOnToggle = l10n.getString("videoPlayerControlCaptionOnToggleButton")
    private val soundOffToggle = l10n.getString("videoPlayerControlAudioOffToggleButton")
    private val videoPlayerReplayTooltip = l10n.getString("videoPlayerReplayTooltip")

    fun visible() = elementContainingTextVisible(closeButton)

    fun closeButtonVisible() = elementContainingTextVisible(closeButton)

    fun subtitlesOnToggleVisible() = elementContainingTextVisible(subtitlesOnToggle)

    fun soundOffToggleVisible() = elementContainingTextVisible(soundOffToggle)

    fun close() = clickElementContainingText(closeButton)

    fun replayButtonVisible() = elementContainingTextVisible(videoPlayerReplayTooltip)


}
