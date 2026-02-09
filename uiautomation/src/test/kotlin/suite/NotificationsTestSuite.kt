package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.notifications.ConfigureNotificationsTests::class,
    feature.notifications.CardNotificationsTests::class,
)
@Suite
@SuiteDisplayName("Notifications Test Suite")
object NotificationsTestSuite
