package helper

import helper.BrowserStackHelper.markTest
import org.junit.jupiter.api.extension.ExtensionContext
import org.junit.jupiter.api.extension.TestWatcher

class TestResultsListener : TestWatcher {

    override fun testSuccessful(extensionContext: ExtensionContext) {
        markTest("passed")
    }

    override fun testAborted(extensionContext: ExtensionContext, throwable: Throwable) {
        markTest("skipped")
    }

    override fun testFailed(extensionContext: ExtensionContext, throwable: Throwable) {
        markTest("failed")
    }
}
