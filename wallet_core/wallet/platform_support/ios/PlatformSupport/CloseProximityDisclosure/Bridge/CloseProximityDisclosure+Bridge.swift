import Foundation

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
