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
        let establishedSessionContext = try establishedSessionContextOrRestartReadTask(session)
        try await sendDeviceResponse(
            session: session,
            establishedSessionContext: establishedSessionContext,
            deviceResponse: deviceResponse
        )

        if isActiveSession(session) {
            await finishSession(session, update: CloseProximityDisclosureUpdate.closed)
        }
    }

    func sendSessionTermination() async throws {
        let session = try requireActiveSession()
        try requireSessionIsActive(session)
        await session.cancelReadMessagesTaskAndWait()
        let establishedSessionContext = try establishedSessionContextOrRestartReadTask(session)
        try await sendSessionTermination(
            session: session,
            transport: establishedSessionContext.transport
        )

        if isActiveSession(session) {
            await finishSession(session, update: CloseProximityDisclosureUpdate.closed)
        }
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
        let advertisedTransports = try await advertiseTransports(buildConnectionMethod())
        let encodedDeviceEngagement = createEncodedDeviceEngagement(
            eDeviceKey: eDeviceKey,
            advertisedTransports: advertisedTransports
        )
        return CloseProximityDisclosureActiveSession(
            channel: channel,
            transports: advertisedTransports,
            eDeviceKey: eDeviceKey,
            encodedDeviceEngagement: encodedDeviceEngagement,
            connectionScope: MainScope()
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

    func advertiseTransports(
        _ connectionMethod: MdocConnectionMethodBle
    ) async throws -> [MdocTransport] {
        try await ConnectionHelperKt.advertise(
            [connectionMethod],
            role: .mdoc,
            transportFactory: MdocTransportFactoryDefault(),
            options: MdocTransportOptions(
                bleUseL2CAP: false,
                bleUseL2CAPInEngagement: false
            )
        )
    }

    func createEncodedDeviceEngagement(
        eDeviceKey: EcPrivateKey,
        advertisedTransports: [MdocTransport]
    ) -> KotlinByteArray {
        let deviceEngagement = buildDeviceEngagement(
            eDeviceKey: eDeviceKey.publicKey,
            version: "1.0"
        ) { builder in
            advertisedTransports.forEach {
                builder.addConnectionMethod(connectionMethod: $0.connectionMethod)
            }
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

            let transport = try await ConnectionHelperKt.waitForConnection(
                session.transports,
                eSenderKey: session.eDeviceKey.publicKey,
                coroutineScope: session.connectionScope
            )
            await handleConnectedTransport(session: session, transport: transport)
        } catch is CancellationError {
            // The connection task is canceled when the session is replaced or stopped.
        } catch {
            guard isActiveSession(session) else { return }
            await failSession(session, error: error)
        }
    }

    func handleConnectedTransport(
        session: CloseProximityDisclosureActiveSession,
        transport: MdocTransport
    ) async {
        guard isActiveSession(session) else {
            await closeStaleTransport(transport)
            return
        }
        do {
            try await session.channel.sendUpdate(update: CloseProximityDisclosureUpdate.connecting)
            session.setTransport(transport)
            startTransportStateObserver(session: session, transport: transport)
            startReadMessagesTask(session)
        } catch {
            guard isActiveSession(session) else { return }
            await failSession(session, error: error)
        }
    }

    func startTransportStateObserver(
        session: CloseProximityDisclosureActiveSession,
        transport: MdocTransport
    ) {
        let transportStateObserverTask = Task { [weak self] in
            guard let self else { return }
            await self.observeTransportState(session: session, transport: transport)
        }
        session.setTransportStateObserverTask(transportStateObserverTask)
    }

    func closeStaleTransport(_ transport: MdocTransport) async {
        // A newer start/stop may already have replaced this session while the connection
        // attempt was in flight. In that case, close the transport we obtained and exit.
        try? await transport.close()
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
                )
            )
        } catch {
            guard isActiveSession(session) else {
                throw error.asCloseProximityDisclosureError
            }
            await failSession(session, error: error)
            throw error.asCloseProximityDisclosureError
        }
    }

    func sendSessionTermination(
        session: CloseProximityDisclosureActiveSession,
        transport: MdocTransport
    ) async throws {
        do {
            try await transport.sendMessage(
                message: SessionEncryption.companion.encodeStatus(
                    statusCode: Int64(Constants.shared.SESSION_DATA_STATUS_SESSION_TERMINATION)
                )
            )
        } catch {
            guard isActiveSession(session) else {
                throw error.asCloseProximityDisclosureError
            }
            await failSession(session, error: error)
            throw error.asCloseProximityDisclosureError
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
