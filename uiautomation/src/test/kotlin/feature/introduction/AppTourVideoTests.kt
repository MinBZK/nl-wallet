package feature.introduction

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertAll
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.introduction.AppTourScreen
import screen.introduction.VideoPlayer
import screen.menu.MenuScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("UC 1.3 App tour video player")
class AppTourVideoTests : TestBase() {

    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var menuScreen: MenuScreen
    private lateinit var appTourScreen: AppTourScreen
    private lateinit var videoPlayer: VideoPlayer

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.Dashboard)
        dashboardScreen = DashboardScreen()
        menuScreen = MenuScreen()
        appTourScreen = AppTourScreen()
        videoPlayer = VideoPlayer()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC14 User views app tour")
    @Tags(Tag("a11yBatch1"))
    fun verifyVideoTour(testInfo: TestInfo) {
        setUp(testInfo)
        assertTrue(dashboardScreen.appTourBannerVisible(), "app tour banner is not visible")

        dashboardScreen.clickMenuButton()
        menuScreen.clickAppTourVideoButton()
        assertAll(
            { assertTrue(appTourScreen.headlineVisible(), "Headline is not visible") },
            { assertTrue(appTourScreen.descriptionVisible(), "Description is not visible") },
            { assertTrue(appTourScreen.videoTitleVisible(), "video title is not visible") },
            { assertTrue(appTourScreen.videoPlayButtonVisible(), "Play button is not visible") },
        )

        appTourScreen.playVideo()
        assertAll(
            { assertTrue(videoPlayer.visible(), "Video player is not visible") },
            { assertTrue(videoPlayer.closeButtonVisible(), "close button is not visible") },
            { assertTrue(videoPlayer.subtitlesOnToggleVisible(), "subtitle on toggle is not visible") },
            { assertTrue(videoPlayer.soundOffToggleVisible(), "sound toggle is not visible") },
        )
        // This sleep is needed (in combination with find element timeout) to ensure that the first video "intro" has finished.
        // This will break when the first video becomes longer
        Thread.sleep(8000)
        assertAll(
            { assertTrue(videoPlayer.subtitlesOnToggleVisible(), "subtitle on toggle is not visible") },
            { assertTrue(videoPlayer.soundOffToggleVisible(), "sound toggle is not visible") },
            { assertTrue(videoPlayer.closeButtonVisible(), "Close button is not visible") },
            { assertTrue(videoPlayer.replayButtonVisible(), "Replay button is not visible") },
        )

        videoPlayer.close()
        assertTrue(appTourScreen.headlineVisible(), "Headline is not visible")
        appTourScreen.clickBackButton()
        assertTrue(menuScreen.menuListButtonsVisible(), "Menu is not visible")
    }
}
