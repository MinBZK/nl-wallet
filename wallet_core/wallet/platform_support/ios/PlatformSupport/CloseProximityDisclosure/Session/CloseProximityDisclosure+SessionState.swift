import Foundation
@preconcurrency import Multipaz

extension CloseProximityDisclosure {
    internal func isBleServerActiveForTesting() -> Bool {
        activeSessionState != nil
    }

    func setActiveSession(_ session: CloseProximityDisclosureActiveSession?) {
        activeSessionState = session.map(CloseProximityDisclosureActiveSessionState.init)
    }

    func takeActiveSessionState() -> CloseProximityDisclosureActiveSessionState? {
        let sessionState = activeSessionState
        activeSessionState = nil
        return sessionState
    }

    func clearActiveSessionIfCurrent(_ session: CloseProximityDisclosureActiveSession) -> Bool {
        guard let activeSessionState, activeSessionState.session === session else { return false }
        self.activeSessionState = nil
        return true
    }

    func isActiveSession(_ session: CloseProximityDisclosureActiveSession) -> Bool {
        activeSessionState?.session === session
    }

    func requireActiveSession() throws -> CloseProximityDisclosureActiveSession {
        guard let session = activeSessionState?.session else {
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

    func setConnectionTask(
        _ connectionTask: Task<Void, Never>,
        for session: CloseProximityDisclosureActiveSession
    ) {
        mutateActiveSessionState(session) { sessionState in
            sessionState.connectionTask = connectionTask
        }
    }

    func setReadMessagesTask(
        _ readMessagesTask: Task<Void, Never>,
        for session: CloseProximityDisclosureActiveSession
    ) {
        mutateActiveSessionState(session) { sessionState in
            sessionState.readMessagesTask = readMessagesTask
        }
    }

    func readerSessionContext(
        for session: CloseProximityDisclosureActiveSession
    ) -> CloseProximityDisclosureReaderSessionContext? {
        guard let activeSessionState, activeSessionState.session === session else { return nil }
        return activeSessionState.readerSessionContext
    }

    func setReaderSessionContext(
        _ readerSessionContext: CloseProximityDisclosureReaderSessionContext,
        for session: CloseProximityDisclosureActiveSession
    ) {
        mutateActiveSessionState(session) { sessionState in
            sessionState.sessionEncryption = readerSessionContext.sessionEncryption
            sessionState.encodedSessionTranscript = readerSessionContext.encodedSessionTranscript
        }
    }

    func establishedSessionContext(
        for session: CloseProximityDisclosureActiveSession
    ) -> CloseProximityDisclosureEstablishedSessionContext? {
        guard let activeSessionState, activeSessionState.session === session else { return nil }
        return activeSessionState.establishedSessionContext
    }

    func cancelReadMessagesTaskAndWait(_ session: CloseProximityDisclosureActiveSession) async {
        let readMessagesTask = takeReadMessagesTask(for: session)
        guard let readMessagesTask else { return }
        readMessagesTask.cancel()
        _ = await readMessagesTask.value
    }

    func cancelBackgroundTasks(_ sessionState: CloseProximityDisclosureActiveSessionState) async {
        sessionState.readMessagesTask?.cancel()
        sessionState.connectionTask?.cancel()

        if let readMessagesTask = sessionState.readMessagesTask {
            _ = await readMessagesTask.value
        }
        if let connectionTask = sessionState.connectionTask {
            _ = await connectionTask.value
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

    private func takeReadMessagesTask(
        for session: CloseProximityDisclosureActiveSession
    ) -> Task<Void, Never>? {
        guard var activeSessionState, activeSessionState.session === session else { return nil }
        let readMessagesTask = activeSessionState.readMessagesTask
        activeSessionState.readMessagesTask = nil
        self.activeSessionState = activeSessionState
        return readMessagesTask
    }

    private func mutateActiveSessionState(
        _ session: CloseProximityDisclosureActiveSession,
        mutation: (inout CloseProximityDisclosureActiveSessionState) -> Void
    ) {
        guard var activeSessionState, activeSessionState.session === session else { return }
        mutation(&activeSessionState)
        self.activeSessionState = activeSessionState
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
