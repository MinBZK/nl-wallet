package nl.rijksoverheid.edi.wallet.platform_support

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope


/**
 * Offload the long running [block] to a separate coroutine dispatcher, to not block the current coroutine thread.
 */
suspend fun <T> longRunning(block: suspend CoroutineScope.() -> T): T =
    coroutineScope { async(Dispatchers.IO) { block() }.await() }
