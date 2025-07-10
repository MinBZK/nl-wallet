package feature.introduction

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertAll
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.introduction.AppTourScreen
import screen.introduction.VideoPlayer
import screen.menu.MenuScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${AppTourVideoTests.USE_CASE} App tour video player [${AppTourVideoTests.JIRA_ID}]")
class AppTourVideoTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 1.3"
        const val JIRA_ID = "PVW-1750"
    }

    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var menuScreen: MenuScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.Dashboard)
        dashboardScreen = DashboardScreen()
        menuScreen = MenuScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.User views apptour video[${JIRA_ID}]")
    fun verifyVideoTour(testInfo: TestInfo) {
        setUp(testInfo)
        Thread.sleep(3000)
        assertTrue(dashboardScreen.appTourBannerVisible(), "app tour banner is not visible")
        dashboardScreen.clickMenuButton()
        menuScreen.clickAppTourVideoButton()
        val appTourScreen = AppTourScreen()
        assertAll(
            { assertTrue(appTourScreen.headlineVisible(), "Headline is not visible") },
            { assertTrue(appTourScreen.descriptionVisible(), "Description is not visible") },
            { assertTrue(appTourScreen.videoTitleVisible(), "video title is not visible") },
            { assertTrue(appTourScreen.videoPlayButtonVisible(), "Play button is not visible") },
        )
        appTourScreen.playVideo()
        val videoPlayer = VideoPlayer()
        Thread.sleep(1000)
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
        assertTrue(menuScreen.visible(), "Menu is not visible")
    }
}
