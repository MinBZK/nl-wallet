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
        let session = try requireActiveSession()
        let _ = try sessionCryptoOrFail(for: session)
        // Stop listening for reader messages at this point, we are just going to write
        await cancelBackgroundTasks(session)
        try requireSessionIsActive(session)
        try await sendDeviceResponse(
            session: session,
            deviceResponse: deviceResponse
        )
        await finishSession(session, update: CloseProximityDisclosureUpdate.closed)
    }

    func sendSessionTermination() async throws {
        let session = try requireActiveSession()
        let _ = try sessionCryptoOrFail(for: session)
        // Stop listening for reader messages at this point, we are just going to write
        await cancelBackgroundTasks(session)
        try requireSessionIsActive(session)
        try await sendSessionTermination(
            session: session
        )
        await finishSession(session, update: CloseProximityDisclosureUpdate.closed)
    }

    func stopBleServer() async throws {
        try await lifecycleLock.withLock { [self] in
            await self.stopBleServerLocked()
        }
    }
}
