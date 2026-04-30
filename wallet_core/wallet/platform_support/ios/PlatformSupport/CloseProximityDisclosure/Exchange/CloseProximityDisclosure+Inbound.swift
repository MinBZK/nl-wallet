import Foundation
@preconcurrency import Multipaz

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
        message: KotlinByteArray
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

    private func waitForSessionMessage(
        session: CloseProximityDisclosureActiveSession
    ) async throws -> KotlinByteArray? {
        let incomingMessage = try await session.transport.waitForMessage()
        guard isActiveSession(session) else { return nil }

        switch incomingMessage {
        case .payload(let message):
            return message.kotlinByteArray()
        case .endOfStream:
            await finishSession(session, update: CloseProximityDisclosureUpdate.closed)
            return nil
        }
    }

    private func createReaderSessionContextFromFirstReaderMessage(
        session: CloseProximityDisclosureActiveSession,
        message: KotlinByteArray
    ) throws -> CloseProximityDisclosureReaderSessionContext {
        let eReaderKey = try SessionEncryption.companion.getEReaderKey(sessionEstablishmentMessage: message)
        let encodedSessionTranscript = buildEncodedSessionTranscript(
            encodedDeviceEngagement: session.encodedDeviceEngagement,
            encodedReaderKey: eReaderKey.encodedCoseKey
        )
        return CloseProximityDisclosureReaderSessionContext(
            sessionEncryption: SessionEncryption(
                role: .mdoc,
                eSelfKey: session.eDeviceKey,
                remotePublicKey: eReaderKey.publicKey,
                encodedSessionTranscript: encodedSessionTranscript
            ),
            encodedSessionTranscript: encodedSessionTranscript
        )
    }

    private func handleReaderMessage(
        session: CloseProximityDisclosureActiveSession,
        message: KotlinByteArray,
        readerSessionContext: CloseProximityDisclosureReaderSessionContext
    ) async throws -> Bool {
        let deviceRequest: KotlinByteArray?
        let status: NSNumber?
        do {
            let decryptedMessage = readerSessionContext.sessionEncryption.decryptMessage(
                messageData: message
            )
            deviceRequest = decryptedMessage.first
            status = decryptedMessage.second
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
                        sessionTranscript: readerSessionContext.encodedSessionTranscript.uint8Array(),
                        deviceRequest: deviceRequest.uint8Array()
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
                message: SessionEncryption.companion.encodeStatus(statusCode: status).uint8Array()
            )
        }
        await failSession(session, error: error)
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
