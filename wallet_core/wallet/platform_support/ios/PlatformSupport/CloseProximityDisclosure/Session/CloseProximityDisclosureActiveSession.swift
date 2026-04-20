import Foundation
@preconcurrency import Multipaz

struct CloseProximityDisclosureSessionState {
    let sessionEncryption: SessionEncryption?
    let encodedSessionTranscript: KotlinByteArray?
}

struct CloseProximityDisclosureEstablishedSessionContext {
    let transport: CloseProximityBleTransport
    let sessionEncryption: SessionEncryption
}

struct CloseProximityDisclosureReaderSessionContext {
    let sessionEncryption: SessionEncryption
    let encodedSessionTranscript: KotlinByteArray
}

final class CloseProximityDisclosureActiveSession {
    private static let backgroundTaskCancellationGracePeriodNanoseconds: UInt64 = 100_000_000

    let channel: CloseProximityDisclosureChannel
    let transport: CloseProximityBleTransport
    let eDeviceKey: EcPrivateKey
    let encodedDeviceEngagement: KotlinByteArray
    // Protects task-handle slots. Connection/read tasks can be installed, canceled, and cleared
    // from different async paths such as startup, send, and shutdown.
    // Example race prevented: sendDeviceResponse() cancels and nils the read-task handle at the
    // same time that startReadMessagesTask() stores a newly created task, losing track of one of
    // the handles and making later shutdown/cancellation inconsistent.
    private let backgroundTasksLock = NSLock()
    // Protects per-session established data written by the inbound read loop and read by outbound
    // send paths. This is separate from activeSessionLock, which only guards which session object
    // is currently active on CloseProximityDisclosure.
    // Example race prevented: the read loop stores sessionEncryption and encodedSessionTranscript
    // while sendDeviceResponse() reads them, observing one field updated and the other still nil.
    private let sessionStateLock = NSLock()
    private var connectionTask: Task<Void, Never>?
    private var readMessagesTask: Task<Void, Never>?
    private var sessionEncryption: SessionEncryption?
    private var encodedSessionTranscript: KotlinByteArray?

    init(
        channel: CloseProximityDisclosureChannel,
        transport: CloseProximityBleTransport,
        eDeviceKey: EcPrivateKey,
        encodedDeviceEngagement: KotlinByteArray
    ) {
        self.channel = channel
        self.transport = transport
        self.eDeviceKey = eDeviceKey
        self.encodedDeviceEngagement = encodedDeviceEngagement
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
            guard let sessionEncryption else { return nil }
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
        tasks.forEach { $0.cancel() }

        for task in tasks {
            await waitForCancellation(of: task)
        }
    }

    private func takeBackgroundTasks() -> [Task<Void, Never>] {
        withBackgroundTasksLock {
            let tasks = [connectionTask, readMessagesTask].compactMap { $0 }
            connectionTask = nil
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
