import Foundation
@preconcurrency import Multipaz

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

struct CloseProximityDisclosureSessionState {
    let sessionEncryption: SessionEncryption?
    let encodedSessionTranscript: KotlinByteArray?
}

struct CloseProximityDisclosureEstablishedSessionContext {
    let transport: MdocTransport
    let sessionEncryption: SessionEncryption
}

struct CloseProximityDisclosureReaderSessionContext {
    let sessionEncryption: SessionEncryption
    let encodedSessionTranscript: KotlinByteArray
}

final class CloseProximityDisclosureActiveSession {
    private static let backgroundTaskCancellationGracePeriodNanoseconds: UInt64 = 100_000_000

    let channel: CloseProximityDisclosureChannel
    let transports: [MdocTransport]
    let eDeviceKey: EcPrivateKey
    let encodedDeviceEngagement: KotlinByteArray
    // This scope is passed into Multipaz waitForConnection(). Canceling it is how we
    // abort the in-flight connection attempt when a session is replaced or stopped.
    let connectionScope: any CoroutineScope
    private let backgroundTasksLock = NSLock()
    private let sessionStateLock = NSLock()
    private var connectionTask: Task<Void, Never>?
    private var transportStateObserverTask: Task<Void, Never>?
    private var readMessagesTask: Task<Void, Never>?
    private var transport: MdocTransport?
    private var sessionEncryption: SessionEncryption?
    private var encodedSessionTranscript: KotlinByteArray?

    init(
        channel: CloseProximityDisclosureChannel,
        transports: [MdocTransport],
        eDeviceKey: EcPrivateKey,
        encodedDeviceEngagement: KotlinByteArray,
        connectionScope: any CoroutineScope
    ) {
        self.channel = channel
        self.transports = transports
        self.eDeviceKey = eDeviceKey
        self.encodedDeviceEngagement = encodedDeviceEngagement
        self.connectionScope = connectionScope
    }

    private func withBackgroundTasksLock<T>(_ body: () -> T) -> T {
        backgroundTasksLock.lock()
        defer { backgroundTasksLock.unlock() }
        return body()
    }

    func setConnectionTask(_ task: Task<Void, Never>) {
        withBackgroundTasksLock {
            connectionTask = task
        }
    }

    func setTransportStateObserverTask(_ task: Task<Void, Never>) {
        withBackgroundTasksLock {
            transportStateObserverTask = task
        }
    }

    func setReadMessagesTask(_ task: Task<Void, Never>) {
        withBackgroundTasksLock {
            readMessagesTask = task
        }
    }

    func cancelReadMessagesTaskAndWait() async {
        let task = withBackgroundTasksLock {
            let task = readMessagesTask
            readMessagesTask = nil
            return task
        }
        task?.cancel()
        if let task {
            await task.value
        }
    }

    private func withSessionStateLock<T>(_ body: () -> T) -> T {
        sessionStateLock.lock()
        defer { sessionStateLock.unlock() }
        return body()
    }

    func setTransport(_ transport: MdocTransport) {
        withSessionStateLock {
            self.transport = transport
        }
    }

    func connectedTransport() -> MdocTransport? {
        withSessionStateLock {
            transport
        }
    }

    func sessionState() -> CloseProximityDisclosureSessionState {
        withSessionStateLock {
            CloseProximityDisclosureSessionState(
                sessionEncryption: sessionEncryption,
                encodedSessionTranscript: encodedSessionTranscript
            )
        }
    }

    func establishedTransportAndEncryption() -> CloseProximityDisclosureEstablishedSessionContext? {
        withSessionStateLock {
            guard let transport, let sessionEncryption else { return nil }
            return CloseProximityDisclosureEstablishedSessionContext(
                transport: transport,
                sessionEncryption: sessionEncryption
            )
        }
    }

    func setSessionEncryption(
        _ sessionEncryption: SessionEncryption,
        encodedSessionTranscript: KotlinByteArray
    ) {
        withSessionStateLock {
            self.sessionEncryption = sessionEncryption
            self.encodedSessionTranscript = encodedSessionTranscript
        }
    }

    func cancelBackgroundTasks() async {
        let tasks = takeBackgroundTasks()
        connectionScope.cancel(cause: nil)
        tasks.forEach { $0.cancel() }

        for task in tasks {
            await waitForCancellation(of: task)
        }
    }

    private func takeBackgroundTasks() -> [Task<Void, Never>] {
        withBackgroundTasksLock {
            let tasks = [connectionTask, transportStateObserverTask, readMessagesTask].compactMap { $0 }
            connectionTask = nil
            transportStateObserverTask = nil
            readMessagesTask = nil
            return tasks
        }
    }

    private func waitForCancellation(of task: Task<Void, Never>) async {
        await withTaskGroup(of: Void.self) { group in
            group.addTask {
                await task.value
            }
            group.addTask {
                try? await Task.sleep(
                    nanoseconds: Self.backgroundTaskCancellationGracePeriodNanoseconds
                )
            }
            await group.next()
            group.cancelAll()
        }
    }
}

extension CloseProximityDisclosureSessionState {
    var readerSessionContext: CloseProximityDisclosureReaderSessionContext? {
        guard let sessionEncryption, let encodedSessionTranscript else { return nil }
        return CloseProximityDisclosureReaderSessionContext(
            sessionEncryption: sessionEncryption,
            encodedSessionTranscript: encodedSessionTranscript
        )
    }
}
