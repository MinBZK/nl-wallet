import Foundation

extension CloseProximityDisclosure: CloseProximityDisclosureBridge {
    func startQrHandover(channel: CloseProximityDisclosureChannel) async throws -> String {
        #if targetEnvironment(simulator)
            throw CloseProximityDisclosureError.PlatformError(
                reason: "Close proximity disclosure is not supported on the iOS Simulator"
            )
        #else
            try await lifecycleLock.withLock { [self] in
                try await self.startQrHandoverLocked(channel: channel)
            }
        #endif
    }

    func sendDeviceResponse(deviceResponse: [UInt8]) async throws {
        let sessionState = try requireActiveSessionState()
        let establishedSessionContext = try establishedSessionContextOrFail(sessionState.session)
        // Stop listening for reader messages at this point, we are just going to write
        await cancelBackgroundTasks(sessionState)
        try requireSessionIsActive(sessionState.session)
        try await sendDeviceResponse(
            session: sessionState.session,
            establishedSessionContext: establishedSessionContext,
            deviceResponse: deviceResponse
        )
        await finishSession(sessionState.session, update: CloseProximityDisclosureUpdate.closed)
    }

    func sendSessionTermination() async throws {
        let sessionState = try requireActiveSessionState()
        let _ = try establishedSessionContextOrFail(sessionState.session)
        // Stop listening for reader messages at this point, we are just going to write
        await cancelBackgroundTasks(sessionState)
        try requireSessionIsActive(sessionState.session)
        try await sendSessionTermination(
            session: sessionState.session
        )
        await finishSession(sessionState.session, update: CloseProximityDisclosureUpdate.closed)
    }

    func stopBleServer() async throws {
        try await lifecycleLock.withLock { [self] in
            await self.stopBleServerLocked()
        }
    }
}
