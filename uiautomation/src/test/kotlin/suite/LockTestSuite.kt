package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.lock.AppLockedStateTests::class,
    feature.lock.UserLocksWalletTests::class,
    feature.lock.UserEntersPinTests::class,
    feature.lock.UserForgetsPinTests::class,
)
@Suite
@SuiteDisplayName("Lock Test Suite")
object LockTestSuite
