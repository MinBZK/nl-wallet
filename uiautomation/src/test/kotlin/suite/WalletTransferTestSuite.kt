package suite

import org.junit.platform.suite.api.SelectClasses
import org.junit.platform.suite.api.Suite
import org.junit.platform.suite.api.SuiteDisplayName

@SelectClasses(
    feature.wallet_transfer.WalletTransferTests::class,
)
@Suite
@SuiteDisplayName("Wallet transfer test suite")
object WalletTransferTestSuite
