import CoreBluetooth
import Foundation

extension CloseProximityDisclosure {
    func startQrHandoverLocked(channel: CloseProximityDisclosureChannel) async throws -> String {
        await stopBleServerLocked()

        do {
            let session = try await createSession(channel: channel)
            setActiveSession(session)
            // Wait for the reader in the background so core can consume the QR immediately.
            startConnectionTask(session)
            return session.encodedDeviceEngagement.base64UrlEncodedString()
        } catch {
            try? await reportStartQrHandoverFailure(channel: channel, error: error)
            throw error.asCloseProximityDisclosureError
        }
    }

    func createSession(
        channel: CloseProximityDisclosureChannel
    ) async throws -> CloseProximityDisclosureActiveSession {
        let serviceUuid = testingPeripheralServerModeUuid ?? UUID()
        let qrSessionSetup = try closeProximityCreateQrSessionSetup(
            peripheralServerUuid: serviceUuid.uint8Array
        )
        let transport = CloseProximityBleTransport(
            serviceUuid: CBUUID(string: serviceUuid.uuidString)
        )
        try await transport.advertise()
        return CloseProximityDisclosureActiveSession(
            channel: channel,
            transport: transport,
            eDevicePrivateKey: qrSessionSetup.eDevicePrivateKey,
            encodedDeviceEngagement: qrSessionSetup.encodedDeviceEngagement
        )
    }

    func startConnectionTask(_ session: CloseProximityDisclosureActiveSession) {
        let connectionTask = Task { [weak self] in
            guard let self else { return }
            do {
                guard await self.isActiveSession(session) else { return }
                try await session.transport.waitForConnection()
                guard await self.isActiveSession(session) else { return }
                try await session.channel.sendUpdate(
                    update: CloseProximityDisclosureUpdate.connected
                )
                try await self.receiveMessages(
                    session: session
                )
            } catch is CancellationError {
                // The connection task is canceled when the session is replaced or stopped.
            } catch {
                guard await self.isActiveSession(session) else { return }
                await self.failSession(session, error: error)
            }
        }
        setConnectionTask(connectionTask, for: session)
    }

    func stopBleServerLocked() async {
        guard let session = takeActiveSession() else { return }

        // Take and clear the session first so any already-running background work immediately
        // observes itself as stale before we start canceling tasks and closing transports.
        await cancelBackgroundTasks(session)
        closeSessionTransports(session)
        try? await session.channel.sendUpdate(
            update: CloseProximityDisclosureUpdate.closed
        )
    }

    func reportStartQrHandoverFailure(
        channel: CloseProximityDisclosureChannel,
        error: Error
    ) async throws {
        try await channel.sendUpdate(
            update: CloseProximityDisclosureUpdate.error(
                error: error.asCloseProximityDisclosureError
            )
        )
    }
}
