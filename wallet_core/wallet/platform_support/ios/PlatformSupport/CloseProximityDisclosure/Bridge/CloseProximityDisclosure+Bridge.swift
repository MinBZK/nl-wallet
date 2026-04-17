import CoreBluetooth
import Foundation
@preconcurrency import Multipaz

extension CloseProximityDisclosure: CloseProximityDisclosureBridge {
    func startQrHandover(channel: CloseProximityDisclosureChannel) async throws -> String {
    #if targetEnvironment(simulator)
        throw CloseProximityDisclosureError.PlatformError(
            reason: "Close proximity disclosure is not supported on the iOS Simulator"
        )
    #else
        try await lifecycleLock.withLock { [self] in
            try await startQrHandoverLocked(channel: channel)
        }
    #endif
    }

    func sendDeviceResponse(deviceResponse: [UInt8]) async throws {
        let session = try requireActiveSession()
        try requireSessionIsActive(session)
        await session.cancelReadMessagesTaskAndWait()
        try requireSessionIsActive(session)
        let establishedSessionContext = try establishedSessionContextOrRestartReadTask(session)
        try await sendDeviceResponse(
            session: session,
            establishedSessionContext: establishedSessionContext,
            deviceResponse: deviceResponse
        )
        await closeSessionAfterDeviceResponse(session)
    }

    func sendSessionTermination() async throws {
        let session = try requireActiveSession()
        try requireSessionIsActive(session)
        await session.cancelReadMessagesTaskAndWait()
        try requireSessionIsActive(session)
        let establishedSessionContext = try establishedSessionContextOrRestartReadTask(session)
        try await sendSessionTermination(
            session: session,
            transport: establishedSessionContext.transport
        )
        await closeSessionAfterDeviceResponse(session)
    }

    func stopBleServer() async throws {
        try await lifecycleLock.withLock { [self] in
            await stopBleServerLocked()
        }
    }
}

extension CloseProximityDisclosure {
    func startQrHandoverLocked(channel: CloseProximityDisclosureChannel) async throws -> String {
        await stopBleServerLocked()

        do {
            let session = try await createSession(channel: channel)
            setActiveSession(session)
            startConnectionTask(session)
            return session.encodedDeviceEngagement.toBase64Url()
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
            serviceUuid: CBUUID(string: String(describing: connectionMethod.peripheralServerModeUuid!))
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
            await self.runConnectionTask(session)
        }
        session.setConnectionTask(connectionTask)
    }

    func runConnectionTask(_ session: CloseProximityDisclosureActiveSession) async {
        do {
            guard isActiveSession(session) else { return }

            try await session.transport.waitForConnection()
            await handleConnectedTransport(session: session)
        } catch is CancellationError {
            // The connection task is canceled when the session is replaced or stopped.
        } catch {
            guard isActiveSession(session) else { return }
            await failSession(session, error: error)
        }
    }

    func handleConnectedTransport(session: CloseProximityDisclosureActiveSession) async {
        guard isActiveSession(session) else { return }
        do {
            try await session.channel.sendUpdate(update: CloseProximityDisclosureUpdate.connecting)
            try await session.channel.sendUpdate(update: CloseProximityDisclosureUpdate.connected)
            startReadMessagesTask(session)
        } catch {
            guard isActiveSession(session) else { return }
            await failSession(session, error: error)
        }
    }

    func reportStartQrHandoverFailure(
        channel: CloseProximityDisclosureChannel,
        error: Error
    ) async throws {
        try await channel.sendUpdate(
            update: CloseProximityDisclosureUpdate.error(error: error.asCloseProximityDisclosureError)
        )
    }

    func requireActiveSession() throws -> CloseProximityDisclosureActiveSession {
        guard let session = withActiveSessionLock({ activeSession }) else {
            throw CloseProximityDisclosureError.PlatformError(
                reason: "No active close proximity disclosure session"
            )
        }
        return session
    }

    func requireSessionIsActive(_ session: CloseProximityDisclosureActiveSession) throws {
        guard isActiveSession(session) else {
            throw CloseProximityDisclosureError.PlatformError(
                reason: "Close proximity disclosure session is no longer active"
            )
        }
    }

    func establishedSessionContextOrRestartReadTask(
        _ session: CloseProximityDisclosureActiveSession
    ) throws -> CloseProximityDisclosureEstablishedSessionContext {
        guard let establishedSessionContext = session.establishedTransportAndEncryption() else {
            if isActiveSession(session) {
                startReadMessagesTask(session)
            }
            throw CloseProximityDisclosureError.PlatformError(
                reason: "Session has not been established yet"
            )
        }
        return establishedSessionContext
    }

    func sendDeviceResponse(
        session: CloseProximityDisclosureActiveSession,
        establishedSessionContext: CloseProximityDisclosureEstablishedSessionContext,
        deviceResponse: [UInt8]
    ) async throws {
        do {
            try await establishedSessionContext.transport.sendMessage(
                message: buildEncryptedDeviceResponse(
                    sessionEncryption: establishedSessionContext.sessionEncryption,
                    deviceResponse: deviceResponse
                ).uint8Array()
            )
        } catch {
            if isActiveSession(session) {
                await failSession(session, error: error)
            }
            throw error.asCloseProximityDisclosureError
        }
    }

    func sendSessionTermination(
        session: CloseProximityDisclosureActiveSession,
        transport: CloseProximityBleTransport
    ) async throws {
        do {
            try await transport.sendMessage(
                message: SessionEncryption.companion.encodeStatus(
                    statusCode: Int64(Constants.shared.SESSION_DATA_STATUS_SESSION_TERMINATION)
                ).uint8Array()
            )
        } catch {
            if isActiveSession(session) {
                await failSession(session, error: error)
            }
            throw error.asCloseProximityDisclosureError
        }
    }

    func closeSessionAfterDeviceResponse(_ session: CloseProximityDisclosureActiveSession) async {
        if isActiveSession(session) {
            await finishSession(session, update: CloseProximityDisclosureUpdate.closed)
        }
    }

    func buildEncryptedDeviceResponse(
        sessionEncryption: SessionEncryption,
        deviceResponse: [UInt8]
    ) -> KotlinByteArray {
        sessionEncryption.encryptMessage(
            messagePlaintext: deviceResponse.kotlinByteArray(),
            statusCode: KotlinLong(
                longLong: Int64(Constants.shared.SESSION_DATA_STATUS_SESSION_TERMINATION)
            )
        )
    }
}
