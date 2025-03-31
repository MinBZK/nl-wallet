package nl.rijksoverheid.edi.wallet.platform_support.utilities

import android.util.Log
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
    block: suspend () -> T): T
{
    var currentDelay = initialDelay
    repeat(times) { index ->

        try {
            Log.d(taskName, taskDescription)
            return block()
        } catch (e: Exception) {
            if (index + 1 == times) {
                Log.e(taskName, "caught ${e.javaClass.name} (description: ${taskDescription}, exception message: \"${e.message?.replace("\n", " ")}\"), giving up..")
                throw e
            }
            Log.w(taskName, "caught ${e.javaClass.name} (description: ${taskDescription}, exception message: \"${e.message?.replace("\n", " ")}\", remaining times: ${times - index}, current delay: ${currentDelay}), retrying..")
        }

        delay(currentDelay)
        currentDelay = currentDelay.times(factor).coerceAtMost(maxDelay)
    }
    throw IllegalStateException("Should not happen")
}
