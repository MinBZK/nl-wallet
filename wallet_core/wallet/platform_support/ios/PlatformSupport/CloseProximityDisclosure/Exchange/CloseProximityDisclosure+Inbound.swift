import Foundation
@preconcurrency import Multipaz

extension CloseProximityDisclosure {
    func receiveMessages(
        session: CloseProximityDisclosureActiveSession,
        transport: CloseProximityBleTransport
    ) async throws {
        while isActiveSession(session) {
            guard let message = try await waitForSessionMessage(
                session: session,
                transport: transport
            ) else {
                return
            }

            let currentReaderSessionContext = readerSessionContext(for: session)
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

    private func createReaderSessionContext(
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
        session: CloseProximityDisclosureActiveSession,
        transport: CloseProximityBleTransport
    ) async throws -> KotlinByteArray? {
        let message = try await transport.waitForMessage().kotlinByteArray()
        guard isActiveSession(session) else { return nil }

        if message.isEmpty {
            await finishSession(session, update: CloseProximityDisclosureUpdate.closed)
            return nil
        }
        return message
    }

    private func ensureReaderSessionContext(
        session: CloseProximityDisclosureActiveSession,
        transport: CloseProximityBleTransport,
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
                await failSessionWithStatus(
                    session,
                    transport: transport,
                    status: status,
                    error: error
                )
                return nil
            }
            throw error
        }
        setReaderSessionContext(newContext, for: session)

        guard isActiveSession(session) else {
            try? await transport.close()
            return nil
        }
        return newContext
    }

    private func handleReaderMessage(
        session: CloseProximityDisclosureActiveSession,
        transport: CloseProximityBleTransport,
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
                    transport: transport,
                    status: status,
                    error: error
                )
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

    private func requireReaderMessageContent(
        deviceRequest: KotlinByteArray?,
        status: NSNumber?
    ) throws {
        if deviceRequest == nil && status == nil {
            throw CloseProximityDisclosureError.PlatformError(
                reason: "Reader message did not contain a device request or status"
            )
        }
    }

    private func sendSessionEstablishedUpdate(
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

    private func failSessionWithStatus(
        _ session: CloseProximityDisclosureActiveSession,
        transport: CloseProximityBleTransport,
        status: Int64,
        error: Error
    ) async {
        // PVW-5710: return the ISO 18013-5 status before shutting BLE down so the reader and
        // wallet core both observe a deterministic close proximity failure.
        if isActiveSession(session) {
            try? await transport.sendMessage(
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
