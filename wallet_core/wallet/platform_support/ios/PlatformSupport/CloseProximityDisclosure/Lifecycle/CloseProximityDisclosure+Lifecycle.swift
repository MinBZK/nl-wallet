import CoreBluetooth
import Foundation
@preconcurrency import Multipaz

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
        let eDeviceKey = Crypto.shared.createEcPrivateKey(curve: .p256)
        let connectionMethod = buildConnectionMethod()
        let transport = CloseProximityBleTransport(
            serviceUuid: CBUUID(
                string: String(describing: connectionMethod.peripheralServerModeUuid!)
            )
        )
        try await transport.advertise()
        let encodedDeviceEngagement = createEncodedDeviceEngagement(
            eDeviceKey: eDeviceKey,
            connectionMethod: connectionMethod
        )
        return CloseProximityDisclosureActiveSession(
            channel: channel,
            transport: transport,
            eDeviceKey: eDeviceKey,
            encodedDeviceEngagement: encodedDeviceEngagement
        )
    }

    func buildConnectionMethod() -> MdocConnectionMethodBle {
        MdocConnectionMethodBle(
            supportsPeripheralServerMode: true,
            supportsCentralClientMode: false,
            peripheralServerModeUuid: testingPeripheralServerModeUuid
                ?? Multipaz.UUID.companion.randomUUID(),
            centralClientModeUuid: nil,
            peripheralServerModePsm: nil,
            // iOS does not expose the local BLE MAC address to apps.
            peripheralServerModeMacAddress: nil
        )
    }

    func createEncodedDeviceEngagement(
        eDeviceKey: EcPrivateKey,
        connectionMethod: MdocConnectionMethodBle
    ) -> KotlinByteArray {
        let deviceEngagement = buildDeviceEngagement(
            eDeviceKey: eDeviceKey.publicKey,
            version: "1.0"
        ) { builder in
            builder.addConnectionMethod(connectionMethod: connectionMethod)
        }
        return Cbor.shared.encode(item: deviceEngagement.toDataItem())
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
        guard let sessionState = takeActiveSessionState() else { return }

        // Take and clear the session first so any already-running background work immediately
        // observes itself as stale before we start canceling tasks and closing transports.
        await cancelBackgroundTasks(sessionState)
        closeSessionTransports(sessionState.session)
        try? await sessionState.session.channel.sendUpdate(
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
