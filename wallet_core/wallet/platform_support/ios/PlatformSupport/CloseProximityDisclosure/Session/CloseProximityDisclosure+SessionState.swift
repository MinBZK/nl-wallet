import Foundation

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
    
    func requireActiveSessionState() throws -> CloseProximityDisclosureActiveSessionState {
        guard let sessionState = activeSessionState else {
            throw CloseProximityDisclosureError.PlatformError(
                reason: "No active close proximity disclosure session"
            )
        }
        return sessionState
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
            sessionState.sessionCrypto = readerSessionContext.sessionCrypto
            sessionState.encodedSessionTranscript = readerSessionContext.encodedSessionTranscript
        }
    }

    func establishedSessionContext(
        for session: CloseProximityDisclosureActiveSession
    ) -> CloseProximityDisclosureEstablishedSessionContext? {
        guard let activeSessionState, activeSessionState.session === session else { return nil }
        return activeSessionState.establishedSessionContext
    }

    // Assumes the session is active
    func establishedSessionContextOrFail(
        _ session: CloseProximityDisclosureActiveSession
    ) throws -> CloseProximityDisclosureEstablishedSessionContext {
        guard let establishedSessionContext = establishedSessionContext(for: session) else {
            throw CloseProximityDisclosureError.PlatformError(
                reason: "Session has not been established yet"
            )
        }
        return establishedSessionContext
    }

    func cancelBackgroundTasks(_ sessionState: CloseProximityDisclosureActiveSessionState) async {
        sessionState.connectionTask?.cancel()

        if let connectionTask = sessionState.connectionTask {
            _ = await connectionTask.value
        }
    }

    func finishSession(
        _ session: CloseProximityDisclosureActiveSession,
        update: CloseProximityDisclosureUpdate
    ) async {
        let shouldReport = clearActiveSessionIfCurrent(session)
        closeSessionTransports(session)
        if shouldReport {
            try? await session.channel.sendUpdate(update: update)
        }
    }

    func failSession(
        _ session: CloseProximityDisclosureActiveSession,
        error: Error
    ) async {
        let shouldReport = clearActiveSessionIfCurrent(session)
        closeSessionTransports(session)
        if shouldReport {
            try? await session.channel.sendUpdate(
                update: CloseProximityDisclosureUpdate.error(
                    error: error.asCloseProximityDisclosureError
                )
            )
        }
    }

    func closeSessionTransports(_ session: CloseProximityDisclosureActiveSession) {
        try? session.transport.close()
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

extension Array where Element == UInt8 {
    func base64UrlEncodedString() -> String {
        Data(self)
            .base64EncodedString()
            .replacingOccurrences(of: "+", with: "-")
            .replacingOccurrences(of: "/", with: "_")
            .replacingOccurrences(of: "=", with: "")
    }
}

extension UUID {
    var uint8Array: [UInt8] {
        withUnsafeBytes(of: uuid) { Array($0) }
    }
}

extension Int64 {
    var asCloseProximityDisclosureUpdate: CloseProximityDisclosureUpdate {
        switch self {
        case CloseProximitySessionStatusCode.termination:
            return .closed
        case CloseProximitySessionStatusCode.sessionEncryptionError:
            return errorUpdate(
                "Reader terminated the session with status 10 (session encryption error)"
            )
        case CloseProximitySessionStatusCode.cborDecodingError:
            return errorUpdate(
                "Reader terminated the session with status 11 (CBOR decoding error)"
            )
        default:
            return errorUpdate(
                "Reader terminated the session with unexpected status \(self)"
            )
        }
    }

    private func errorUpdate(_ reason: String) -> CloseProximityDisclosureUpdate {
        .error(error: .PlatformError(reason: reason))
    }
}
