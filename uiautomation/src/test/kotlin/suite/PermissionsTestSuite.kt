package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.permissions.BluetoothPermissionTests::class,
    feature.permissions.CameraPermissionTests::class,
    feature.permissions.NotificationPermissionTests::class,
)
@Suite
@SuiteDisplayName("Permissions Test Suite")
object PermissionsTestSuite
