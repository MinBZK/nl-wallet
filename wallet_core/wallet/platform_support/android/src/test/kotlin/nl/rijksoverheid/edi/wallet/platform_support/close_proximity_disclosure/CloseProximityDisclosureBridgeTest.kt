package nl.rijksoverheid.edi.wallet.platform_support.close_proximity_disclosure

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import org.multipaz.mdoc.transport.MdocTransport
import org.multipaz.mdoc.transport.MdocTransportClosedException

class CloseProximityDisclosureBridgeTest {
    @Test
    fun `transport state classifier reports closed for closed and failed states`() {
        assertTrue(transportStateShouldReportClosedUpdate(MdocTransport.State.CLOSED))
        assertTrue(transportStateShouldReportClosedUpdate(MdocTransport.State.FAILED))
    }

    @Test
    fun `transport state classifier ignores non-terminal states`() {
        assertFalse(transportStateShouldReportClosedUpdate(MdocTransport.State.CONNECTED))
        assertFalse(transportStateShouldReportClosedUpdate(MdocTransport.State.CONNECTING))
        assertFalse(transportStateShouldReportClosedUpdate(MdocTransport.State.ADVERTISING))
    }

    @Test
    fun `transport failure classifier reports closed for transport closed exceptions`() {
        assertTrue(transportFailureShouldReportClosedUpdate(MdocTransportClosedException("transport closed")))
    }

    @Test
    fun `transport failure classifier ignores non-transport exceptions`() {
        assertFalse(transportFailureShouldReportClosedUpdate(IllegalStateException("bad state")))
        assertFalse(transportFailureShouldReportClosedUpdate(RuntimeException("boom")))
    }
}
