import Foundation

extension CloseProximityDisclosure {

    func sendDeviceResponse(
        session: CloseProximityDisclosureActiveSession,
        deviceResponse: [UInt8]
    ) async throws {
        do {
            try await session.transport.sendMessage(
                message: buildEncryptedDeviceResponse(
                    session: session,
                    deviceResponse: deviceResponse
                )
            )
        } catch {
            if isActiveSession(session) {
                await finishSessionOnDisconnectOrFail(session, error: error)
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
                await finishSessionOnDisconnectOrFail(session, error: error)
            }
            throw error.asCloseProximityDisclosureError
        }
    }

    private func buildEncryptedDeviceResponse(
        session: CloseProximityDisclosureActiveSession,
        deviceResponse: [UInt8]
    ) throws -> [UInt8] {
        try sessionCryptoOrFail(for: session).encrypt(
            plaintext: deviceResponse,
            statusCode: CloseProximitySessionStatusCode.termination
        )
    }
}
