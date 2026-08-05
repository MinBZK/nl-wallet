package feature.notifications

import helper.LocalizationHelper
import helper.TasDataHelper
import helper.TestBase
import navigator.MenuNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junit.jupiter.api.assertAll
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.issuance.CardIssuanceScreen
import screen.issuance.DisclosureIssuanceScreen
import screen.menu.MenuScreen
import screen.security.PinScreen
import screen.settings.NotificationsDebugScreen
import screen.settings.NotificationsDebugScreen.CardNotificationType.EXPIRED
import screen.settings.NotificationsDebugScreen.CardNotificationType.EXPIRES_SOON
import screen.settings.NotificationsScreen
import screen.settings.SettingsScreen
import screen.web.demo.DemoIndexWebPage
import screen.web.demo.issuer.IssuerWebPage
import java.time.LocalDate
import java.time.LocalDateTime
import java.time.ZoneOffset
import java.time.format.DateTimeFormatter

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("Card notifications")
class CardNotificationsTests : TestBase() {

    private lateinit var indexWebPage: DemoIndexWebPage
    private lateinit var issuerWebPage: IssuerWebPage
    private lateinit var disclosureForIssuanceScreen: DisclosureIssuanceScreen
    private lateinit var cardIssuanceScreen: CardIssuanceScreen
    private lateinit var pinScreen: PinScreen
    private lateinit var l10n: LocalizationHelper
    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var menuScreen: MenuScreen
    private lateinit var settingsScreen: SettingsScreen
    private lateinit var notificationsScreen: NotificationsScreen
    private lateinit var notificationsDebugScreen: NotificationsDebugScreen
    private lateinit var tasData: TasDataHelper

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        pinScreen = PinScreen()
        l10n = LocalizationHelper()
        disclosureForIssuanceScreen = DisclosureIssuanceScreen()
        cardIssuanceScreen = CardIssuanceScreen()
        dashboardScreen = DashboardScreen()
        menuScreen = MenuScreen()
        settingsScreen = SettingsScreen()
        notificationsScreen = NotificationsScreen()
        notificationsDebugScreen = NotificationsDebugScreen()
        tasData = TasDataHelper()
        indexWebPage = DemoIndexWebPage()
        issuerWebPage = IssuerWebPage()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC71 System schedules notifications for card status changes")
    fun verifyCardNotificationSchedules(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        MenuScreen().clickBrowserTestButton()
        indexWebPage.switchToWebViewContext()
        indexWebPage.clickLoyaltyButton()
        issuerWebPage.openSameDeviceWalletFlow()
        issuerWebPage.acceptOpenWalletDialog()

        disclosureForIssuanceScreen.switchToNativeContext()
        cardIssuanceScreen.clickAddCardButton()
        pinScreen.enterPin(DEFAULT_PIN)
        cardIssuanceScreen.clickToDashboardButton()

        dashboardScreen.clickMenuButton()
        menuScreen.clickSettingsButton()
        settingsScreen.clickNotificationsButton()
        notificationsScreen.toggleNotifications()
        notificationsScreen.clickDebugScreenButton()

        val pidExpiresSoonTimer = notificationsDebugScreen.getCardNotificationTimer(tasData.getPidDisplayName(), EXPIRES_SOON)
        val pidExpiresSoonVisible = notificationsDebugScreen.isNotificationVisible(tasData.getPidDisplayName(), EXPIRES_SOON)
        val pidExpiresSoonChannel = notificationsDebugScreen.getCardNotificationChannel(tasData.getPidDisplayName(), EXPIRES_SOON)
        val pidExpiresSoonId = notificationsDebugScreen.getCardNotificationID(tasData.getPidDisplayName(), EXPIRES_SOON)

        val pidExpiredTimer = notificationsDebugScreen.getCardNotificationTimer(tasData.getPidDisplayName(), EXPIRED)
        val pidExpiredVisible = notificationsDebugScreen.isNotificationVisible(tasData.getPidDisplayName(), EXPIRED)
        val pidExpiredChannel = notificationsDebugScreen.getCardNotificationChannel(tasData.getPidDisplayName(), EXPIRED)
        val pidExpiredId = notificationsDebugScreen.getCardNotificationID(tasData.getPidDisplayName(), EXPIRED)

        val loyaltyExpiresSoonTimer = notificationsDebugScreen.getCardNotificationTimer(tasData.getLoyaltyDisplayName(), EXPIRES_SOON)
        val loyaltyExpiresSoonVisible = notificationsDebugScreen.isNotificationVisible(tasData.getLoyaltyDisplayName(), EXPIRES_SOON)
        val loyaltyExpiresSoonChannel = notificationsDebugScreen.getCardNotificationChannel(tasData.getLoyaltyDisplayName(), EXPIRES_SOON)
        val loyaltyExpiresSoonId = notificationsDebugScreen.getCardNotificationID(tasData.getLoyaltyDisplayName(), EXPIRES_SOON)

        val loyaltyExpiredTimer = notificationsDebugScreen.getCardNotificationTimer(tasData.getLoyaltyDisplayName(), EXPIRED)
        val loyaltyExpiredVisible = notificationsDebugScreen.isNotificationVisible(tasData.getLoyaltyDisplayName(), EXPIRED)
        val loyaltyExpiredChannel = notificationsDebugScreen.getCardNotificationChannel(tasData.getLoyaltyDisplayName(), EXPIRED)
        val loyaltyExpiredId = notificationsDebugScreen.getCardNotificationID(tasData.getLoyaltyDisplayName(), EXPIRED)

        assertAll(
            { assertTrue(pidExpiresSoonVisible, "Notification text is not visible for PID expires soon notification") },
            { assertTrue(pidExpiresSoonChannel.contains("cardUpdates"), "Incorrect notification channel for PID expires soon notification") },
            { assertTrue(pidExpiresSoonId.toIntOrNull() != null, "Incorrect notification id for PID expires soon notification") },
            { assertTrue(verifyDateIsOneYearMinusDaysfromNow(pidExpiresSoonTimer, 7), "Incorrect timer for PID expires soon notification") },

            { assertTrue(pidExpiredVisible, "Notification text is not visible for PID expired notification") },
            { assertTrue(pidExpiredChannel.contains("cardUpdates"), "Incorrect notification channel for PID expired notification") },
            { assertTrue(pidExpiredId.toIntOrNull() != null, "Incorrect notification id for PID expired notification") },
            { assertTrue(verifyDateIsOneYearFromNow(pidExpiredTimer), "Incorrect timer for PID expired notification") },

            { assertTrue(loyaltyExpiresSoonVisible, "Notification text is not visible for Loyalty card expires soon notification") },
            { assertTrue(loyaltyExpiresSoonChannel.contains("cardUpdates"), "Incorrect notification channel for Loyalty card expires soon notification") },
            { assertTrue(loyaltyExpiresSoonId.toIntOrNull() != null, "Incorrect notification Id for Loyalty card expires soon notification") },
            { assertTrue(verifyDateIsOneYearMinusDaysfromNow(loyaltyExpiresSoonTimer, 7), "Incorrect timer for Loyalty card expires soon notification") },

            { assertTrue(loyaltyExpiredVisible, "Notification text is not visible for Loyalty card expired notification") },
            { assertTrue(loyaltyExpiredChannel.contains("cardUpdates"), "Incorrect notification channel for Loyalty card expired notification") },
            { assertTrue(loyaltyExpiredId.toIntOrNull() != null, "Incorrect notification id for Loyalty card expired notification") },
            { assertTrue(verifyDateIsOneYearFromNow(loyaltyExpiredTimer), "Incorrect timer for Loyalty card expired notification") },
        )
    }

    private fun isDateCorrect(dateString: String, expectedDate: LocalDate): Boolean {
        val formatter = DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm:ss.SSS")
        val parsedDateTime = LocalDateTime.parse(dateString, formatter)
        return parsedDateTime.toLocalDate() == expectedDate
    }

    private fun verifyDateIsOneYearMinusDaysfromNow(dateString: String, days: Long): Boolean {
        val expected = LocalDate.now(ZoneOffset.UTC).plusYears(1).minusDays(days)
        return isDateCorrect(dateString, expected)
    }

    private fun verifyDateIsOneYearFromNow(dateString: String): Boolean {
        val expected = LocalDate.now(ZoneOffset.UTC).plusYears(1)
        return isDateCorrect(dateString, expected)
    }
}
