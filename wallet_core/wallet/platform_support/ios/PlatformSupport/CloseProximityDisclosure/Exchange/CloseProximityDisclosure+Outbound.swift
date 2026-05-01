import Foundation

extension CloseProximityDisclosure {

    func sendDeviceResponse(
        session: CloseProximityDisclosureActiveSession,
        establishedSessionContext: CloseProximityDisclosureEstablishedSessionContext,
        deviceResponse: [UInt8]
    ) async throws {
        do {
            try await establishedSessionContext.transport.sendMessage(
                message: buildEncryptedDeviceResponse(
                    sessionCrypto: establishedSessionContext.sessionCrypto,
                    deviceResponse: deviceResponse
                )
            )
        } catch {
            if isActiveSession(session) {
                await failSession(session, error: error)
            }
            throw error.asCloseProximityDisclosureError
        }
    }

    func sendSessionTermination(
        session: CloseProximityDisclosureActiveSession
    ) async throws {
        do {
            try await session.transport.sendMessage(
                message: try closeProximityEncodeSessionStatus(
                    statusCode: CloseProximitySessionStatusCode.termination
                )
            )
        } catch {
            if isActiveSession(session) {
                await failSession(session, error: error)
            }
            throw error.asCloseProximityDisclosureError
        }
    }

    private func buildEncryptedDeviceResponse(
        sessionCrypto: CloseProximitySessionCrypto,
        deviceResponse: [UInt8]
    ) throws -> [UInt8] {
        try sessionCrypto.encrypt(
            plaintext: deviceResponse,
            statusCode: CloseProximitySessionStatusCode.termination
        )
    }
}
