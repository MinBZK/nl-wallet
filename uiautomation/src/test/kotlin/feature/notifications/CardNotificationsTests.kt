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
import java.time.ZoneId
import java.time.ZoneOffset
import java.time.ZonedDateTime
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
        indexWebPage.clickInsuranceButton()
        issuerWebPage.openSameDeviceWalletFlow()
        disclosureForIssuanceScreen.switchToNativeContext()
        disclosureForIssuanceScreen.share()
        pinScreen.enterPin(DEFAULT_PIN)
        cardIssuanceScreen.clickAddCardButton()
        pinScreen.enterPin(DEFAULT_PIN)
        cardIssuanceScreen.clickToDashboardButton()

        dashboardScreen.clickMenuButton()
        menuScreen.clickSettingsButton()
        settingsScreen.clickNotificationsButton()
        notificationsScreen.toggleNotifications()
        notificationsScreen.clickDebugScreenButton()

        val pidExpiresSoonNotificationTimer = notificationsDebugScreen.getCardNotificationTimer(
            tasData.getPidDisplayName(), EXPIRES_SOON)
        val pidExpiredNotificationTimer = notificationsDebugScreen.getCardNotificationTimer(
            tasData.getPidDisplayName(), EXPIRED)
        val insuranceExpiresSoonNotificationTimer = notificationsDebugScreen.getCardNotificationTimer(
            tasData.getInsuranceDisplayName(), EXPIRES_SOON)
        val insuranceExpiredNotificationTimer = notificationsDebugScreen.getCardNotificationTimer(tasData.getInsuranceDisplayName(),
            EXPIRED)

        assertAll(
            { assertTrue(notificationsDebugScreen.isNotificationVisible(tasData.getPidDisplayName(),
                EXPIRES_SOON), "Notification text is not visible for PID expires soon notification") },
            { assertTrue(notificationsDebugScreen.getCardNotificationChannel(tasData.getPidDisplayName(),
                EXPIRES_SOON).contains("cardUpdates"), "Incorrect notification channel for PID expires soon notification") },
            { assertTrue(notificationsDebugScreen.getCardNotificationID(tasData.getPidDisplayName(),
                EXPIRES_SOON).toIntOrNull() != null, "Incorrect notification id for PID expires soon notification") },
            { assertTrue( verifyDateIsOneYearMinusDaysfromNow(pidExpiresSoonNotificationTimer, 7), "Incorrect timer for PID expires soon notification") },

            { assertTrue(notificationsDebugScreen.isNotificationVisible(tasData.getPidDisplayName(),
                EXPIRED), "Notification text is not visible for PID expired notification") },
            { assertTrue(notificationsDebugScreen.getCardNotificationChannel(tasData.getPidDisplayName(),
                EXPIRED).contains("cardUpdates"), "Incorrect notification channel for PID expired notification") },
            { assertTrue(notificationsDebugScreen.getCardNotificationID(tasData.getPidDisplayName(),
                EXPIRED).toIntOrNull() != null, "Incorrect notification id for PID expired notification") },
            { assertTrue( verifyDateIsOneYearFromNow(pidExpiredNotificationTimer), "Incorrect timer for PID expired notification") },

            { assertTrue(notificationsDebugScreen.isNotificationVisible(tasData.getInsuranceDisplayName(),
                EXPIRES_SOON), "Notification text is not visible for insurance expires soon notification") },
            { assertTrue(notificationsDebugScreen.getCardNotificationChannel(tasData.getInsuranceDisplayName(),
                EXPIRES_SOON).contains("cardUpdates"), "Incorrect notification channel for insurance expires soon notification") },
            { assertTrue(notificationsDebugScreen.getCardNotificationID(tasData.getInsuranceDisplayName(),
                EXPIRES_SOON).toIntOrNull() != null, "Incorrect notification id for insurance expires soon notification") },
            { assertTrue( verifyDateIsOneYearMinusDaysfromNow(insuranceExpiresSoonNotificationTimer, 7), "Incorrect timer for insurance expires soon notification") },

            { assertTrue(notificationsDebugScreen.isNotificationVisible(tasData.getInsuranceDisplayName(),
                EXPIRED), "Notification text is not visible for insurance expired notification") },
            { assertTrue(notificationsDebugScreen.getCardNotificationChannel(tasData.getInsuranceDisplayName(),
                EXPIRED).contains("cardUpdates"), "Incorrect notification channel for insurance expired notification") },
            { assertTrue(notificationsDebugScreen.getCardNotificationID(tasData.getInsuranceDisplayName(),
                EXPIRED).toIntOrNull() != null, "Incorrect notification id for insurance expired notification") },
            { assertTrue( verifyDateIsOneYearFromNow(insuranceExpiredNotificationTimer), "Incorrect timer for insurance expired notification") },
        )
    }

    private fun isDateCorrect(dateString: String, expectedDate: LocalDate): Boolean {
        val formatter = DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm:ss.SSS")
        val parsedDateTime = LocalDateTime.parse(dateString, formatter)

        val zonedDateTime = ZonedDateTime.of(parsedDateTime, ZoneId.of("Europe/Amsterdam"))
        val utcDateTime = zonedDateTime.toInstant().atZone(ZoneOffset.UTC)

        val isCorrectDate = utcDateTime.toLocalDate() == expectedDate
        val isCorrectTime = utcDateTime.hour == 0 && utcDateTime.minute == 0

        return isCorrectDate && isCorrectTime
    }

    private fun verifyDateIsOneYearMinusDaysfromNow(dateString: String, days: Long): Boolean {
        val expected = LocalDate.now().plusYears(1).minusDays(days)
        return isDateCorrect(dateString, expected)
    }

    private fun verifyDateIsOneYearFromNow(dateString: String): Boolean {
        val expected = LocalDate.now().plusYears(1)
        return isDateCorrect(dateString, expected)
    }
}
