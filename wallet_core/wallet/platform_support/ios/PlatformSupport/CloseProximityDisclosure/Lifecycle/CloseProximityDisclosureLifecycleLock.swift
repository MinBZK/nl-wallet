import Foundation

// Serializes lifecycle transactions across await points. An actor is used instead of NSLock
// because the guarded start/stop operations suspend while the critical section is still active.
// Example race prevented: stopBleServer() begins closing a session, then startQrHandover() slips
// in during an await and creates a new session before the old teardown has actually finished.
actor CloseProximityDisclosureLifecycleLock {
    private struct Waiter {
        let id: Foundation.UUID
        let continuation: CheckedContinuation<Void, Error>
    }

    private var isLocked = false
    private var waiters: [Waiter] = []

    func withLock<T: Sendable>(_ body: @Sendable () async throws -> T) async throws -> T {
        try await acquire()
        defer { release() }

        try Task.checkCancellation()
        return try await body()
    }

    private func acquire() async throws {
        try Task.checkCancellation()

        guard isLocked else {
            isLocked = true
            return
        }

        let waiterId = Foundation.UUID()

        try await withTaskCancellationHandler {
            try await withCheckedThrowingContinuation { continuation in
                waiters.append(Waiter(id: waiterId, continuation: continuation))
            }
        } onCancel: {
            Task {
                await self.cancelWaiter(id: waiterId)
            }
        }
    }

    private func cancelWaiter(id: Foundation.UUID) {
        guard let waiterIndex = waiters.firstIndex(where: { $0.id == id }) else { return }

        let waiter = waiters.remove(at: waiterIndex)
        waiter.continuation.resume(throwing: CancellationError())
    }

    private func release() {
        guard !waiters.isEmpty else {
            isLocked = false
            return
        }

        waiters.removeFirst().continuation.resume(returning: ())
    }
}
