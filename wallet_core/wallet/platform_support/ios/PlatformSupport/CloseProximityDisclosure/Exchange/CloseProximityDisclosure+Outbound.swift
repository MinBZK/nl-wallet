import Foundation
@preconcurrency import Multipaz

extension CloseProximityDisclosure {
    func establishedSessionContextOrRestartReadTask(
        _ session: CloseProximityDisclosureActiveSession
    ) throws -> CloseProximityDisclosureEstablishedSessionContext {
        try requireSessionIsActive(session)
        guard let establishedSessionContext = establishedSessionContext(for: session) else {
            startReadMessagesTask(session)
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

    private func buildEncryptedDeviceResponse(
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
