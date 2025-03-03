package nl.rijksoverheid.edi.wallet.platform_support.utilities

import android.app.ActivityManager.TaskDescription
import android.util.Log
import kotlinx.coroutines.delay

suspend fun <T> retryable(
    times: Int = 10,
    initialDelay: Long = 3000,
    maxDelay: Long = 120000,
    factor: Double = 2.0,
    taskName: String = "retry",
    taskDescription: String = "retryable-task",
    block: suspend () -> T): T
{
    var currentDelay = initialDelay
    var remainingTimes = times
    repeat(times - 1) {

        try {
            Log.d(taskName, taskDescription)
            return block()
        } catch (e: Exception) {
            Log.d(taskName, "caught ${e.javaClass.name} (description: $taskDescription, remaining times: $remainingTimes, current delay: $currentDelay)")
        }

        delay(currentDelay)
        currentDelay = (currentDelay * factor).toLong().coerceAtMost(maxDelay)
        remainingTimes = (remainingTimes -1)
    }
    return block()
}
