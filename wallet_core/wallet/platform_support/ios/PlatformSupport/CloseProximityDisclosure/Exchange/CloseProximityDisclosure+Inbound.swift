import Foundation

extension CloseProximityDisclosure {
    func receiveMessages(
        session: CloseProximityDisclosureActiveSession
    ) async throws {
        while isActiveSession(session) {
            guard let message = try await waitForSessionMessage(
                session: session
            ) else {
                return
            }
            guard isActiveSession(session) else { return }
            // According to the ISO-18013-5 protocol, the reader will send the eReader public key as the first payload.
            // This is the last ingredient needed for the session transcript.
            guard let readerSessionContext = await ensureReaderSessionContext(
                session: session,
                message: message
            ) else {
                return
            }
            
            guard isActiveSession(session) else { return }

            if try await handleReaderMessage(
                session: session,
                message: message,
                readerSessionContext: readerSessionContext
            ) {
                return
            }
        }
    }

    private func ensureReaderSessionContext(
        session: CloseProximityDisclosureActiveSession,
        message: [UInt8]
    ) async -> CloseProximityDisclosureReaderSessionContext? {
        if let readerSessionContext = readerSessionContext(for: session) {
            return readerSessionContext
        }

        do {
            let establishedReaderSessionContext = try createReaderSessionContextFromFirstReaderMessage(
                session: session,
                message: message
            )
            setReaderSessionContext(establishedReaderSessionContext, for: session)
            return self.readerSessionContext(for: session)
        } catch {
            if let status = sessionEstablishmentFailureStatus(for: error) {
                await failSessionWithStatus(session, status: status, error: error)
            } else {
                await failSession(session, error: error)
            }
            return nil
        }
    }

    private func buildEncodedSessionTranscript(
        encodedDeviceEngagement: [UInt8],
        encodedReaderKey: [UInt8]
    ) throws -> [UInt8] {
        try closeProximityBuildSessionTranscript(
            encodedDeviceEngagement: encodedDeviceEngagement,
            encodedReaderKey: encodedReaderKey
        )
    }

    private func waitForSessionMessage(
        session: CloseProximityDisclosureActiveSession
    ) async throws -> [UInt8]? {
        let incomingMessage = try await session.transport.waitForMessage()

        switch incomingMessage {
        case .payload(let message):
            return message
        case .endOfStream:
            await finishSession(session, update: CloseProximityDisclosureUpdate.closed)
            return nil
        }
    }

    private func createReaderSessionContextFromFirstReaderMessage(
        session: CloseProximityDisclosureActiveSession,
        message: [UInt8]
    ) throws -> CloseProximityDisclosureReaderSessionContext {
        let eReaderKey = try closeProximityGetEReaderKey(sessionEstablishmentMessage: message)
        let encodedSessionTranscript = try buildEncodedSessionTranscript(
            encodedDeviceEngagement: session.encodedDeviceEngagement,
            encodedReaderKey: eReaderKey.encodedCoseKey
        )
        return CloseProximityDisclosureReaderSessionContext(
            sessionCrypto: try CloseProximitySessionCrypto(
                eDevicePrivateKey: session.eDevicePrivateKey,
                encodedReaderKey: eReaderKey.encodedCoseKey,
                encodedSessionTranscript: encodedSessionTranscript
            ),
            encodedSessionTranscript: encodedSessionTranscript
        )
    }

    private func handleReaderMessage(
        session: CloseProximityDisclosureActiveSession,
        message: [UInt8],
        readerSessionContext: CloseProximityDisclosureReaderSessionContext
    ) async throws -> Bool {
        let deviceRequest: [UInt8]?
        let status: Int64?
        do {
            let decryptedMessage = try readerSessionContext.sessionCrypto.decrypt(message: message)
            deviceRequest = decryptedMessage.data
            status = decryptedMessage.status
        } catch {
            if let status = sessionMessageFailureStatus(for: error) {
                await failSessionWithStatus(
                    session,
                    status: status,
                    error: error
                )
                return true
            }
            throw error
        }

        if deviceRequest == nil && status == nil {
            throw CloseProximityDisclosureError.PlatformError(
                reason: "Reader message did not contain a device request or status"
            )
        }
        if let deviceRequest {
            if isActiveSession(session) {
                try await session.channel.sendUpdate(
                    update: CloseProximityDisclosureUpdate.sessionEstablished(
                        sessionTranscript: readerSessionContext.encodedSessionTranscript,
                        deviceRequest: deviceRequest
                    )
                )
            }
        }
        if let status {
            await finishSession(session, update: status.asCloseProximityDisclosureUpdate)
            return true
        }
        return false
    }

    private func failSessionWithStatus(
        _ session: CloseProximityDisclosureActiveSession,
        status: Int64,
        error: Error
    ) async {
        // PVW-5710: return the ISO 18013-5 status before shutting BLE down so the reader and
        // wallet core both observe a deterministic close proximity failure.
        if isActiveSession(session) {
            try? await session.transport.sendMessage(
                message: try closeProximityEncodeSessionStatus(statusCode: status)
            )
        }
        await failSession(session, error: error)
    }
}

private func sessionEstablishmentFailureStatus(for error: Error) -> Int64? {
    (error as? CloseProximitySessionCryptoError)?.closeProximityStatusCode
}

private func sessionMessageFailureStatus(for error: Error) -> Int64? {
    (error as? CloseProximitySessionCryptoError)?.closeProximityStatusCode
}

private extension CloseProximitySessionCryptoError {
    var closeProximityStatusCode: Int64 {
        switch self {
        case .CborDecoding:
            CloseProximitySessionStatusCode.cborDecodingError
        case .SessionEncryption, .Other:
            CloseProximitySessionStatusCode.sessionEncryptionError
        }
    }
}
