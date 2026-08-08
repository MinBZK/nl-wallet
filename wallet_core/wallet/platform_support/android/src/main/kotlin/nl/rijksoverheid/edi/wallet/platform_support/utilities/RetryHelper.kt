package nl.rijksoverheid.edi.wallet.platform_support.utilities

import kotlinx.coroutines.delay
import kotlin.time.Duration
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.seconds

suspend fun <T> retryable(
    times: Int = 10,
    initialDelay: Duration = 100.milliseconds,
    maxDelay: Duration = 4.seconds,
    factor: Double = 2.0,
    taskName: String = "retry",
    taskDescription: String = "retryable-task",
    nonRetryable: (Exception) -> Boolean = { false },
    block: suspend () -> T): T
{
    var currentDelay = initialDelay
    var attempt = 0
    while (true) {
        PlatformLogger.d(taskName, taskDescription)
        attempt++
        try {
            return block()
        } catch (e: Exception) {
            // Some failures are permanent (e.g. the device has no Google Play services at all);
            // retrying those only wastes time, so rethrow them immediately.
            if (nonRetryable(e)) {
                Log.i(taskName, "caught non-retryable ${e.javaClass.name} (description: ${taskDescription}, exception message: \"${e.message}\"), not retrying..")
                throw e
            }
            if (attempt == times) {
                PlatformLogger.e(taskName, "caught ${e.javaClass.name} (description: ${taskDescription}, exception message: \"${e.message}\"), giving up..")
                throw e

            }
            PlatformLogger.w(taskName, "caught ${e.javaClass.name} (description: ${taskDescription}, exception message: \"${e.message}\", remaining times: ${times - attempt}, current delay: ${currentDelay}), retrying..")
        }

        delay(currentDelay)
        currentDelay = currentDelay.times(factor).coerceAtMost(maxDelay)
    }
}
