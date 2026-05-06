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
            let sessionCryptoBundle = await handleFirstReaderMessage(
                session: session,
                message: message
            )
            if let sessionCryptoBundle {
                guard isActiveSession(session) else { return }

                if try await handleReaderMessage(
                    session: session,
                    message: message,
                    sessionCrypto: sessionCryptoBundle.0,
                    encodedSessionTranscript: sessionCryptoBundle.1
                ) {
                    return
                }
            }
        }
    }

    private func handleFirstReaderMessage(
        session: CloseProximityDisclosureActiveSession,
        message: [UInt8]
    ) async -> (CloseProximitySessionCrypto, [UInt8])? {
        // If there is session encryption, then this is not the first reader message so skip
        let existingSessionCrypto = sessionCrypto(for: session)
        let existingSessionTranscript = session.encodedSessionTranscript
        if let existingSessionCrypto, let existingSessionTranscript {
            return (existingSessionCrypto, existingSessionTranscript)
        }
        // If there is no session encryption, try to treat this message as the reader setting up the session,
        // and fail the whole session if this doesn't work as we won't get this message again
        do {
            let (sessionCrypto, encodedSessionTranscript) = try createSessionCryptoFromFirstReaderMessage(
                session: session,
                message: message
            )
            setSessionCrypto(
                sessionCrypto,
                encodedSessionTranscript: encodedSessionTranscript,
                for: session
            )
            return (sessionCrypto, encodedSessionTranscript)
        } catch {
            if let status = mapErrorToSessionStatus(for: error) {
                await failSessionWithStatus(session, status: status, error: error)
            } else {
                await failSession(session, error: error)
            }
            return nil
        }
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

    private func createSessionCryptoFromFirstReaderMessage(
        session: CloseProximityDisclosureActiveSession,
        message: [UInt8]
    ) throws -> (CloseProximitySessionCrypto, [UInt8]) {
        let eReaderKey = try closeProximityGetEReaderKey(sessionEstablishmentMessage: message)
        let encodedSessionTranscript = try closeProximityBuildSessionTranscript(
            encodedDeviceEngagement: session.encodedDeviceEngagement,
            encodedReaderKey: eReaderKey.encodedCoseKey
        )
        return (
            try CloseProximitySessionCrypto(
                eDevicePrivateKey: session.eDevicePrivateKey,
                encodedReaderKey: eReaderKey.encodedCoseKey,
                encodedSessionTranscript: encodedSessionTranscript
            ),
            encodedSessionTranscript
        )
    }

    private func handleReaderMessage(
        session: CloseProximityDisclosureActiveSession,
        message: [UInt8],
        sessionCrypto: CloseProximitySessionCrypto,
        encodedSessionTranscript: [UInt8]
    ) async throws -> Bool {
        let deviceRequest: [UInt8]?
        let status: Int64?
        do {
            let decryptedMessage = try sessionCrypto.decrypt(message: message)
            deviceRequest = decryptedMessage.data
            status = decryptedMessage.status
        } catch {
            if let status = mapErrorToSessionStatus(for: error) {
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
                        sessionTranscript: encodedSessionTranscript,
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

private func mapErrorToSessionStatus(for error: Error) -> Int64? {
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
