import Foundation

extension CloseProximityDisclosure {
    internal func isBleServerActiveForTesting() -> Bool {
        activeSession != nil
    }

    func setActiveSession(_ session: CloseProximityDisclosureActiveSession?) {
        activeSession = session
    }

    func takeActiveSession() -> CloseProximityDisclosureActiveSession? {
        let session = activeSession
        activeSession = nil
        return session
    }

    func clearActiveSessionIfCurrent(_ session: CloseProximityDisclosureActiveSession) -> Bool {
        guard let activeSession, activeSession === session else { return false }
        self.activeSession = nil
        return true
    }

    func isActiveSession(_ session: CloseProximityDisclosureActiveSession) -> Bool {
        activeSession === session
    }
    
    func requireActiveSession() throws -> CloseProximityDisclosureActiveSession {
        guard let session = activeSession else {
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
        mutateActiveSession(session) { session in
            session.connectionTask = connectionTask
        }
    }

    func sessionCrypto(
        for session: CloseProximityDisclosureActiveSession
    ) -> CloseProximitySessionCrypto? {
        guard let activeSession, activeSession === session else { return nil }
        return activeSession.sessionCrypto
    }

    func encodedSessionTranscript(
        for session: CloseProximityDisclosureActiveSession
    ) -> [UInt8]? {
        guard let activeSession, activeSession === session else { return nil }
        return activeSession.encodedSessionTranscript
    }

    func setSessionCrypto(
        _ sessionCrypto: CloseProximitySessionCrypto,
        encodedSessionTranscript: [UInt8],
        for session: CloseProximityDisclosureActiveSession
    ) {
        mutateActiveSession(session) { session in
            session.sessionCrypto = sessionCrypto
            session.encodedSessionTranscript = encodedSessionTranscript
        }
    }

    // Assumes the session is active
    func sessionCryptoOrFail(
        for session: CloseProximityDisclosureActiveSession
    ) throws -> CloseProximitySessionCrypto {
        guard let sessionCrypto = sessionCrypto(for: session) else {
            throw CloseProximityDisclosureError.PlatformError(
                reason: "Session has not been established yet"
            )
        }
        return sessionCrypto
    }

    // Assumes the session is active
    func encodedSessionTranscriptOrFail(
        for session: CloseProximityDisclosureActiveSession
    ) throws -> [UInt8] {
        guard let encodedSessionTranscript = encodedSessionTranscript(for: session) else {
            throw CloseProximityDisclosureError.PlatformError(
                reason: "Session transcript missing after session establishment"
            )
        }
        return encodedSessionTranscript
    }

    func cancelBackgroundTasks(_ session: CloseProximityDisclosureActiveSession) async {
        session.connectionTask?.cancel()

        if let connectionTask = session.connectionTask {
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

    private func mutateActiveSession(
        _ session: CloseProximityDisclosureActiveSession,
        mutation: (CloseProximityDisclosureActiveSession) -> Void
    ) {
        guard let activeSession, activeSession === session else { return }
        mutation(activeSession)
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
