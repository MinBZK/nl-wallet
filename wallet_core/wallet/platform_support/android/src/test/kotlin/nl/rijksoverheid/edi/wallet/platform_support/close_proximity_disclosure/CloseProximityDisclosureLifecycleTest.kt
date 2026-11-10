package nl.rijksoverheid.edi.wallet.platform_support.close_proximity_disclosure

import android.content.Context
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkObject
import io.mockk.unmockkObject
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.job
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.multipaz.mdoc.connectionmethod.MdocConnectionMethod
import org.multipaz.mdoc.transport.MdocTransport
import org.multipaz.mdoc.transport.MdocTransportFactory
import org.multipaz.util.Logger
import uniffi.platform_support.CloseProximityDisclosureChannel
import uniffi.platform_support.CloseProximityDisclosureUpdate

@OptIn(ExperimentalCoroutinesApi::class)
class CloseProximityDisclosureLifecycleTest {
    private class PendingConnection(method: MdocConnectionMethod) {
        val opened = CompletableDeferred<Unit>()
        var openJob: Job? = null
        var cancelledBeforeClose = false
        private val transportState = MutableStateFlow(MdocTransport.State.IDLE)
        val transport = mockk<MdocTransport> {
            every { connectionMethod } returns method
            every { state } returns transportState
            coEvery { advertise() } coAnswers {
                transportState.value = MdocTransport.State.ADVERTISING
            }
            coEvery { open(any()) } coAnswers {
                openJob = currentCoroutineContext().job
                opened.complete(Unit)
                awaitCancellation()
            }
            coEvery { close() } coAnswers {
                cancelledBeforeClose = openJob?.isCompleted == true
                transportState.value = MdocTransport.State.CLOSED
            }
        }
    }

    private val connections = mutableListOf<PendingConnection>()

    @Before
    fun setUp() {
        // Multipaz logs connection attempts through android.util.Log, which is unavailable on the JVM.
        mockkObject(Logger)
        every { Logger.i(any(), any()) } returns Unit
        every { Logger.e(any(), any(), any()) } returns Unit
    }

    @After
    fun tearDown() = runTest {
        // Also clean up detached Multipaz jobs when a regression makes an assertion fail.
        connections.forEach { it.openJob?.cancelAndJoin() }
        unmockkObject(Logger)
    }

    private fun TestScope.createBridge(): CloseProximityDisclosureBridge {
        val context = mockk<Context>()
        every { context.applicationContext } returns context
        val factory = mockk<MdocTransportFactory> {
            every { createTransport(any(), any(), any()) } answers {
                PendingConnection(firstArg()).also { connections.add(it) }.transport
            }
        }
        return CloseProximityDisclosureBridge(context, transportFactory = factory, bridgeScope = backgroundScope)
    }

    @Test
    fun `stopping an unconnected session cancels opening before closing the transport`() = runTest {
        val bridge = createBridge()
        val channel = mockk<CloseProximityDisclosureChannel>(relaxed = true)

        bridge.startQrHandover(channel)
        val connection = connections.single()
        connection.opened.await()
        bridge.stopBleServer()

        assertTrue(connection.cancelledBeforeClose)
        assertFalse(bridge.isBleServerActiveForTesting())
        coVerify(exactly = 1) { channel.sendUpdate(CloseProximityDisclosureUpdate.Closed) }
        coVerify(exactly = 0) { channel.sendUpdate(ofType<CloseProximityDisclosureUpdate.Error>()) }
    }

    @Test
    fun `replacing an unconnected session cancels its connection work`() = runTest {
        val bridge = createBridge()
        val firstChannel = mockk<CloseProximityDisclosureChannel>(relaxed = true)
        val secondChannel = mockk<CloseProximityDisclosureChannel>(relaxed = true)

        bridge.startQrHandover(firstChannel)
        val firstConnection = connections.single()
        firstConnection.opened.await()

        bridge.startQrHandover(secondChannel)
        val secondConnection = connections.last()
        secondConnection.opened.await()

        assertTrue(firstConnection.cancelledBeforeClose)
        assertTrue(bridge.isBleServerActiveForTesting())
        assertTrue(secondConnection.openJob?.isActive == true)
        coVerify(exactly = 1) { firstChannel.sendUpdate(CloseProximityDisclosureUpdate.Closed) }
        coVerify(exactly = 0) { secondChannel.sendUpdate(any()) }

        bridge.stopBleServer()
        assertTrue(secondConnection.cancelledBeforeClose)
        coVerify(exactly = 1) { secondChannel.sendUpdate(CloseProximityDisclosureUpdate.Closed) }
    }

    @Test
    fun `stopping immediately does not open a closed transport`() = runTest {
        val bridge = createBridge()
        val channel = mockk<CloseProximityDisclosureChannel>(relaxed = true)

        bridge.startQrHandover(channel)
        bridge.stopBleServer()
        runCurrent()

        coVerify(exactly = 0) { connections.single().transport.open(any()) }
        coVerify(exactly = 1) { connections.single().transport.close() }
        coVerify(exactly = 1) { channel.sendUpdate(CloseProximityDisclosureUpdate.Closed) }
    }
}
