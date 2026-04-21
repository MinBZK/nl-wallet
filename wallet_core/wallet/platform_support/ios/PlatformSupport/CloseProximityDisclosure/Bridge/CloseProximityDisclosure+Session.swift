import Foundation
@preconcurrency import Multipaz

extension CloseProximityDisclosure {
    internal func isBleServerActiveForTesting() -> Bool {
        withActiveSessionLock {
            activeSession != nil
        }
    }

    func withActiveSessionLock<T>(_ body: () -> T) -> T {
        activeSessionLock.lock()
        defer { activeSessionLock.unlock() }
        return body()
    }

    func setActiveSession(_ session: CloseProximityDisclosureActiveSession?) {
        withActiveSessionLock {
            activeSession = session
        }
    }

    func takeActiveSession() -> CloseProximityDisclosureActiveSession? {
        withActiveSessionLock {
            let session = activeSession
            activeSession = nil
            return session
        }
    }

    func clearActiveSessionIfCurrent(_ session: CloseProximityDisclosureActiveSession) -> Bool {
        withActiveSessionLock {
            if activeSession === session {
                activeSession = nil
                return true
            }
            return false
        }
    }

    func isActiveSession(_ session: CloseProximityDisclosureActiveSession) -> Bool {
        withActiveSessionLock {
            activeSession === session
        }
    }

    func createReaderSessionContext(
        eDeviceKey: EcPrivateKey,
        encodedDeviceEngagement: KotlinByteArray,
        message: KotlinByteArray
    ) -> CloseProximityDisclosureReaderSessionContext {
        let eReaderKey = SessionEncryption.companion.getEReaderKey(
            sessionEstablishmentMessage: message
        )
        let encodedSessionTranscript = buildEncodedSessionTranscript(
            encodedDeviceEngagement: encodedDeviceEngagement,
            encodedReaderKey: eReaderKey.encodedCoseKey
        )
        return CloseProximityDisclosureReaderSessionContext(
            sessionEncryption: SessionEncryption(
                role: .mdoc,
                eSelfKey: eDeviceKey,
                remotePublicKey: eReaderKey.publicKey,
                encodedSessionTranscript: encodedSessionTranscript
            ),
            encodedSessionTranscript: encodedSessionTranscript
        )
    }

    func buildEncodedSessionTranscript(
        encodedDeviceEngagement: KotlinByteArray,
        encodedReaderKey: KotlinByteArray
    ) -> KotlinByteArray {
        Cbor.shared.encode(
            item: CborArrayKt.buildCborArray { builder in
                builder.add(
                    item: Tagged(
                        tagNumber: Tagged.companion.ENCODED_CBOR,
                        taggedItem: Bstr(value: encodedDeviceEngagement)
                    )
                )
                builder.add(
                    item: Tagged(
                        tagNumber: Tagged.companion.ENCODED_CBOR,
                        taggedItem: Bstr(value: encodedReaderKey)
                    )
                )
                builder.add(item: Simple.companion.NULL)
            }
        )
    }

    func observeTransportState(
        session: CloseProximityDisclosureActiveSession,
        transport: MdocTransport
    ) async {
        do {
            for await state in transport.state {
                guard isActiveSession(session) else { return }
                try await handleTransportStateUpdate(session: session, state: state)
            }
        } catch is CancellationError {
            // Expected when the session is replaced or stopped and the observer task is canceled.
        } catch {
            guard isActiveSession(session) else { return }
            await failSession(session, error: error)
        }
    }

    func handleTransportStateUpdate(
        session: CloseProximityDisclosureActiveSession,
        state: MdocTransport.State
    ) async throws {
        switch state {
        case .connected:
            try await session.channel.sendUpdate(update: CloseProximityDisclosureUpdate.connected)
        case .closed:
            await finishSession(session, update: CloseProximityDisclosureUpdate.closed)
        case .failed:
            // Transport failures are expected to surface through the active connect/read/write
            // operation as well, and that path is responsible for failSession().
            return
        default:
            return
        }
    }

    func receiveMessages(
        session: CloseProximityDisclosureActiveSession,
        transport: MdocTransport
    ) async throws {
        while isActiveSession(session) {
            guard let message = try await waitForSessionMessage(session: session, transport: transport) else {
                return
            }
            let currentReaderSessionContext = session.sessionState().readerSessionContext
            guard let readerSessionContext = try await ensureReaderSessionContext(
                session: session,
                transport: transport,
                message: message,
                currentContext: currentReaderSessionContext
            ) else {
                return
            }
            if try await handleReaderMessage(
                session: session,
                transport: transport,
                message: message,
                readerSessionContext: readerSessionContext
            ) {
                return
            }
        }
    }

    func waitForSessionMessage(
        session: CloseProximityDisclosureActiveSession,
        transport: MdocTransport
    ) async throws -> KotlinByteArray? {
        let message = try await transport.waitForMessage()
        guard isActiveSession(session) else { return nil }

        if message.isEmpty {
            await finishSession(session, update: CloseProximityDisclosureUpdate.closed)
            return nil
        }
        return message
    }

    func ensureReaderSessionContext(
        session: CloseProximityDisclosureActiveSession,
        transport: MdocTransport,
        message: KotlinByteArray,
        currentContext: CloseProximityDisclosureReaderSessionContext?
    ) async throws -> CloseProximityDisclosureReaderSessionContext? {
        if let currentContext {
            return currentContext
        }

        let newContext: CloseProximityDisclosureReaderSessionContext
        do {
            newContext = createReaderSessionContext(
                eDeviceKey: session.eDeviceKey,
                encodedDeviceEngagement: session.encodedDeviceEngagement,
                message: message
            )
        } catch {
            if let status = sessionEstablishmentFailureStatus(for: error) {
                await failSessionWithStatus(session, transport: transport, status: status, error: error)
                return nil
            }
            throw error
        }
        session.setSessionEncryption(
            newContext.sessionEncryption,
            encodedSessionTranscript: newContext.encodedSessionTranscript
        )

        guard isActiveSession(session) else {
            try? await transport.close()
            return nil
        }
        return newContext
    }

    func handleReaderMessage(
        session: CloseProximityDisclosureActiveSession,
        transport: MdocTransport,
        message: KotlinByteArray,
        readerSessionContext: CloseProximityDisclosureReaderSessionContext
    ) async throws -> Bool {
        let deviceRequest: KotlinByteArray?
        let status: NSNumber?
        do {
            let decryptedMessage = readerSessionContext.sessionEncryption.decryptMessage(messageData: message)
            deviceRequest = decryptedMessage.first
            status = decryptedMessage.second
        } catch {
            if let status = sessionMessageFailureStatus(for: error) {
                await failSessionWithStatus(session, transport: transport, status: status, error: error)
                return true
            }
            throw error
        }

        try requireReaderMessageContent(deviceRequest: deviceRequest, status: status)
        if let deviceRequest {
            try await sendSessionEstablishedUpdate(
                session: session,
                encodedSessionTranscript: readerSessionContext.encodedSessionTranscript,
                deviceRequest: deviceRequest
            )
        }
        if let status {
            await finishSession(session, update: status.asCloseProximityDisclosureUpdate)
            return true
        }
        return false
    }

    func requireReaderMessageContent(
        deviceRequest: KotlinByteArray?,
        status: NSNumber?
    ) throws {
        if deviceRequest == nil && status == nil {
            throw CloseProximityDisclosureError.PlatformError(
                reason: "Reader message did not contain a device request or status"
            )
        }
    }

    func sendSessionEstablishedUpdate(
        session: CloseProximityDisclosureActiveSession,
        encodedSessionTranscript: KotlinByteArray,
        deviceRequest: KotlinByteArray
    ) async throws {
        try await session.channel.sendUpdate(
            update: CloseProximityDisclosureUpdate.sessionEstablished(
                sessionTranscript: encodedSessionTranscript.uint8Array(),
                deviceRequest: deviceRequest.uint8Array()
            )
        )
    }

    func finishSession(
        _ session: CloseProximityDisclosureActiveSession,
        update: CloseProximityDisclosureUpdate
    ) async {
        let shouldReport = clearActiveSessionIfCurrent(session)
        await closeSessionTransports(session)
        if shouldReport {
            try? await session.channel.sendUpdate(update: update)
        }
    }

    func failSession(
        _ session: CloseProximityDisclosureActiveSession,
        error: Error
    ) async {
        let shouldReport = clearActiveSessionIfCurrent(session)
        await closeSessionTransports(session)
        if shouldReport {
            try? await session.channel.sendUpdate(
                update: CloseProximityDisclosureUpdate.error(
                    error: error.asCloseProximityDisclosureError
                )
            )
        }
    }

    func failSessionWithStatus(
        _ session: CloseProximityDisclosureActiveSession,
        transport: MdocTransport,
        status: Int64,
        error: Error
    ) async {
        // PVW-5710: return the ISO 18013-5 status before shutting BLE down so the reader and
        // wallet core both observe a deterministic close proximity failure.
        if isActiveSession(session) {
            try? await transport.sendMessage(
                message: SessionEncryption.companion.encodeStatus(statusCode: status)
            )
        }
        await failSession(session, error: error)
    }

    func stopBleServerLocked() async {
        guard let session = takeActiveSession() else { return }

        // Take and clear the session first so any already-running background work immediately observes
        // itself as stale before we start canceling tasks and closing transports.
        await session.cancelReadMessagesTaskAndWait()
        await session.cancelBackgroundTasks()
        await closeSessionTransports(session)
        try? await session.channel.sendUpdate(update: CloseProximityDisclosureUpdate.closed)
    }

    func startReadMessagesTask(_ session: CloseProximityDisclosureActiveSession) {
        guard let transport = session.connectedTransport() else { return }

        let readMessagesTask = Task { [weak self] in
            guard let self else { return }
            await self.runReadMessagesTask(session: session, transport: transport)
        }
        session.setReadMessagesTask(readMessagesTask)
    }

    func runReadMessagesTask(
        session: CloseProximityDisclosureActiveSession,
        transport: MdocTransport
    ) async {
        do {
            try await receiveMessages(
                session: session,
                transport: transport
            )
        } catch is CancellationError {
            // The read task is canceled explicitly while shutting the session down, so nothing
            // needs to be reported from this path.
        } catch {
            guard isActiveSession(session) else { return }
            await failSession(session, error: error)
        }
    }

    func closeSessionTransports(_ session: CloseProximityDisclosureActiveSession) async {
        for transport in session.transports {
            try? await transport.close()
        }
    }
}

extension Error {
    var asCloseProximityDisclosureError: CloseProximityDisclosureError {
        if let error = self as? CloseProximityDisclosureError {
            return error
        }
        return .from(self)
    }
}

private func sessionEstablishmentFailureStatus(for error: Error) -> Int64? {
    switch error {
    case is KotlinIllegalArgumentException, is KotlinIllegalStateException:
        return Int64(Constants.shared.SESSION_DATA_STATUS_ERROR_CBOR_DECODING)
    default:
        return nil
    }
}

private func sessionMessageFailureStatus(for error: Error) -> Int64? {
    switch error {
    case is KotlinIllegalArgumentException:
        return Int64(Constants.shared.SESSION_DATA_STATUS_ERROR_CBOR_DECODING)
    case is KotlinIllegalStateException:
        return Int64(Constants.shared.SESSION_DATA_STATUS_ERROR_SESSION_ENCRYPTION)
    default:
        return nil
    }
}

extension KotlinByteArray {
    var isEmpty: Bool {
        size == 0
    }

    func uint8Array() -> [UInt8] {
        (0..<Int(size)).map { index in
            UInt8(bitPattern: get(index: Int32(index)))
        }
    }
}

extension Array where Element == UInt8 {
    func kotlinByteArray() -> KotlinByteArray {
        let byteArray = KotlinByteArray(size: Int32(count))
        for (index, byte) in enumerated() {
            byteArray.set(index: Int32(index), value: Int8(bitPattern: byte))
        }
        return byteArray
    }
}

extension NSNumber {
    var asCloseProximityDisclosureUpdate: CloseProximityDisclosureUpdate {
        switch int64Value {
        case Constants.shared.SESSION_DATA_STATUS_SESSION_TERMINATION:
            return .closed
        case Constants.shared.SESSION_DATA_STATUS_ERROR_SESSION_ENCRYPTION:
            return errorUpdate(
                "Reader terminated the session with status 10 (session encryption error)"
            )
        case Constants.shared.SESSION_DATA_STATUS_ERROR_CBOR_DECODING:
            return errorUpdate(
                "Reader terminated the session with status 11 (CBOR decoding error)"
            )
        default:
            return errorUpdate(
                "Reader terminated the session with unexpected status \(int64Value)"
            )
        }
    }

    private func errorUpdate(_ reason: String) -> CloseProximityDisclosureUpdate {
        .error(error: .PlatformError(reason: reason))
    }
}
