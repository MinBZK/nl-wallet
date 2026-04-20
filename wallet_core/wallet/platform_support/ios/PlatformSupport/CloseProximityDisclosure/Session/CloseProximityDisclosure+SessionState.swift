import Foundation
@preconcurrency import Multipaz

extension CloseProximityDisclosure {
    internal func isBleServerActiveForTesting() -> Bool {
        withActiveSessionLock {
            activeSession != nil
        }
    }

    func withActiveSessionLock<T>(_ body: () -> T) -> T {
        activeSessionLock.lock()
        defer { activeSessionLock.unlock() }
        return body()
    }

    func setActiveSession(_ session: CloseProximityDisclosureActiveSession?) {
        withActiveSessionLock {
            activeSession = session
        }
    }

    func takeActiveSession() -> CloseProximityDisclosureActiveSession? {
        withActiveSessionLock {
            let session = activeSession
            activeSession = nil
            return session
        }
    }

    func clearActiveSessionIfCurrent(_ session: CloseProximityDisclosureActiveSession) -> Bool {
        withActiveSessionLock {
            if activeSession === session {
                activeSession = nil
                return true
            }
            return false
        }
    }

    func isActiveSession(_ session: CloseProximityDisclosureActiveSession) -> Bool {
        withActiveSessionLock {
            activeSession === session
        }
    }

    func requireActiveSession() throws -> CloseProximityDisclosureActiveSession {
        guard let session = withActiveSessionLock({ activeSession }) else {
            throw CloseProximityDisclosureError.PlatformError(
                reason: "No active close proximity disclosure session"
            )
        }
        return session
    }

    func requireSessionIsActive(_ session: CloseProximityDisclosureActiveSession) throws {
        guard isActiveSession(session) else {
            throw CloseProximityDisclosureError.PlatformError(
                reason: "Close proximity disclosure session is no longer active"
            )
        }
    }

    func finishSession(
        _ session: CloseProximityDisclosureActiveSession,
        update: CloseProximityDisclosureUpdate
    ) async {
        let shouldReport = clearActiveSessionIfCurrent(session)
        await closeSessionTransports(session)
        if shouldReport {
            try? await session.channel.sendUpdate(update: update)
        }
    }

    func failSession(
        _ session: CloseProximityDisclosureActiveSession,
        error: Error
    ) async {
        let shouldReport = clearActiveSessionIfCurrent(session)
        await closeSessionTransports(session)
        if shouldReport {
            try? await session.channel.sendUpdate(
                update: CloseProximityDisclosureUpdate.error(
                    error: error.asCloseProximityDisclosureError
                )
            )
        }
    }

    func closeSessionTransports(_ session: CloseProximityDisclosureActiveSession) async {
        try? await session.transport.close()
    }
}

extension Error {
    var asCloseProximityDisclosureError: CloseProximityDisclosureError {
        if let error = self as? CloseProximityDisclosureError {
            return error
        }
        return .from(self)
    }
}

extension KotlinByteArray {
    var isEmpty: Bool {
        size == 0
    }

    func uint8Array() -> [UInt8] {
        (0..<Int(size)).map { index in
            UInt8(bitPattern: get(index: Int32(index)))
        }
    }
}

extension Array where Element == UInt8 {
    func kotlinByteArray() -> KotlinByteArray {
        let byteArray = KotlinByteArray(size: Int32(count))
        for (index, byte) in enumerated() {
            byteArray.set(index: Int32(index), value: Int8(bitPattern: byte))
        }
        return byteArray
    }
}

extension NSNumber {
    var asCloseProximityDisclosureUpdate: CloseProximityDisclosureUpdate {
        switch int64Value {
        case Constants.shared.SESSION_DATA_STATUS_SESSION_TERMINATION:
            return .closed
        case Constants.shared.SESSION_DATA_STATUS_ERROR_SESSION_ENCRYPTION:
            return errorUpdate(
                "Reader terminated the session with status 10 (session encryption error)"
            )
        case Constants.shared.SESSION_DATA_STATUS_ERROR_CBOR_DECODING:
            return errorUpdate(
                "Reader terminated the session with status 11 (CBOR decoding error)"
            )
        default:
            return errorUpdate(
                "Reader terminated the session with unexpected status \(int64Value)"
            )
        }
    }

    private func errorUpdate(_ reason: String) -> CloseProximityDisclosureUpdate {
        .error(error: .PlatformError(reason: reason))
    }
}
