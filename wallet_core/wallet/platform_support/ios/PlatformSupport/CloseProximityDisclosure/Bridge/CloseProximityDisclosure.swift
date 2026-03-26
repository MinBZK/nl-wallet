//
//  CloseProximityDisclosure.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 06/03/2026.
//

import Foundation
@preconcurrency import Multipaz

final class CloseProximityDisclosure: @unchecked Sendable {
    // UniFFI/Rust can reach these async bridge methods from unrelated tasks. Serialize
    // lifecycle transitions so rapid start/stop/start calls behave deterministically,
    // and make the queue cancellation-aware so canceled waiters do not wake up later
    // and interfere with a newer session.
    private actor LifecycleLock {
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

    private final class ActiveSession {
        private static let backgroundTaskCancellationGracePeriodNanoseconds: UInt64 = 100_000_000

        let channel: CloseProximityDisclosureChannel
        let transports: [MdocTransport]
        // This scope is passed into Multipaz waitForConnection(). Canceling it is how we
        // abort the in-flight connection attempt when a session is replaced or stopped.
        let connectionScope: any CoroutineScope
        private let backgroundTasksLock = NSLock()
        private var connectionTask: Task<Void, Never>?
        private var transportStateObserverTask: Task<Void, Never>?

        init(
            channel: CloseProximityDisclosureChannel,
            transports: [MdocTransport],
            connectionScope: any CoroutineScope
        ) {
            self.channel = channel
            self.transports = transports
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

        func cancelBackgroundTasks() async {
            let tasks = withBackgroundTasksLock {
                let tasks = [connectionTask, transportStateObserverTask].compactMap { $0 }
                connectionTask = nil
                transportStateObserverTask = nil
                return tasks
            }

            // Cancel both the Kotlin scope and the outer Swift tasks before closing transports.
            // That makes an old session go stale first and gives Multipaz a chance to unwind any
            // in-flight wait/open work before transport.close() runs.
            connectionScope.cancel(cause: nil)
            tasks.forEach { $0.cancel() }

            for task in tasks {
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
    }

    private let activeSessionLock = NSLock()
    private let lifecycleLock = LifecycleLock()
    private let testingPeripheralServerModeUuid: Multipaz.UUID?
    // Background tasks only act on the session they were created for. Identity checks against the
    // current activeSession prevent stale work from a replaced session from emitting updates after
    // a newer handover has already started.
    private var activeSession: ActiveSession?

    init(testingPeripheralServerModeUuid: Multipaz.UUID? = nil) {
        self.testingPeripheralServerModeUuid = testingPeripheralServerModeUuid
    }

    private func withActiveSessionLock<T>(_ body: () -> T) -> T {
        activeSessionLock.lock()
        defer { activeSessionLock.unlock() }
        return body()
    }

    private func setActiveSession(_ session: ActiveSession?) {
        withActiveSessionLock {
            activeSession = session
        }
    }

    private func takeActiveSession() -> ActiveSession? {
        withActiveSessionLock {
            let session = activeSession
            activeSession = nil
            return session
        }
    }

    internal func isBleServerActiveForTesting() -> Bool {
        withActiveSessionLock {
            activeSession != nil
        }
    }

    private func clearActiveSessionIfCurrent(_ session: ActiveSession) -> Bool {
        withActiveSessionLock {
            if activeSession === session {
                activeSession = nil
                return true
            }
            return false
        }
    }

    private func isActiveSession(_ session: ActiveSession) -> Bool {
        withActiveSessionLock {
            activeSession === session
        }
    }

    private func createSessionEncryption(
        eDeviceKey: EcPrivateKey,
        encodedDeviceEngagement: KotlinByteArray,
        message: KotlinByteArray
    ) -> (sessionEncryption: SessionEncryption, encodedSessionTranscript: KotlinByteArray) {
        let eReaderKey = SessionEncryption.companion.getEReaderKey(
            sessionEstablishmentMessage: message
        )
        let encodedSessionTranscript = Cbor.shared.encode(
            item: CborArrayKt.buildCborArray { builder in
                builder.add(
                    item: Tagged(
                        tagNumber: Tagged.companion.ENCODED_CBOR,
                        taggedItem: Bstr(value: encodedDeviceEngagement)
                    )
                )
                builder.add(
                    item: Tagged(
                        tagNumber: Tagged.companion.ENCODED_CBOR,
                        taggedItem: Bstr(value: eReaderKey.encodedCoseKey)
                    )
                )
                builder.add(item: Simple.companion.NULL)
            }
        )
        return (
            sessionEncryption: SessionEncryption(
                role: .mdoc,
                eSelfKey: eDeviceKey,
                remotePublicKey: eReaderKey.publicKey,
                encodedSessionTranscript: encodedSessionTranscript
            ),
            encodedSessionTranscript: encodedSessionTranscript
        )
    }

    private func observeTransportState(
        session: ActiveSession,
        transport: MdocTransport
    ) async {
        do {
            for await state in transport.state {
                guard isActiveSession(session) else { return }

                switch state {
                case .connected:
                    try await session.channel.sendUpdate(
                        update: CloseProximityDisclosureUpdate.connected)
                case .closed:
                    await finishSession(session, update: CloseProximityDisclosureUpdate.closed)
                    return
                case .failed:
                    return
                default:
                    continue
                }
            }
        } catch is CancellationError {
            // No errors are needed on cancellation, as this is triggered from the cancelWaiter
            // and core is assumed to clean up it's own state
        } catch {
            guard isActiveSession(session) else { return }
            await failSession(session, error: error)
        }
    }

    private func receiveMessages(
        session: ActiveSession,
        transport: MdocTransport,
        eDeviceKey: EcPrivateKey,
        encodedDeviceEngagement: KotlinByteArray
    ) async throws {
        var sessionEncryption: SessionEncryption?
        var encodedSessionTranscript: KotlinByteArray?

        while isActiveSession(session) {
            let message = try await transport.waitForMessage()
            guard isActiveSession(session) else { return }

            if message.isEmpty {
                await finishSession(session, update: CloseProximityDisclosureUpdate.closed)
                return
            }

            if sessionEncryption == nil || encodedSessionTranscript == nil {
                let sessionState = createSessionEncryption(
                    eDeviceKey: eDeviceKey,
                    encodedDeviceEngagement: encodedDeviceEngagement,
                    message: message
                )
                sessionEncryption = sessionState.sessionEncryption
                encodedSessionTranscript = sessionState.encodedSessionTranscript

                if !isActiveSession(session) {
                    try? await transport.close()
                    return
                }
            }

            let decryptedMessage = sessionEncryption!.decryptMessage(messageData: message)
            let deviceRequest = decryptedMessage.first
            let status = decryptedMessage.second

            if deviceRequest == nil && status == nil {
                throw CloseProximityDisclosureError.PlatformError(
                    reason: "Reader message did not contain a device request or status"
                )
            }

            if let deviceRequest {
                try await session.channel.sendUpdate(
                    update: CloseProximityDisclosureUpdate.sessionEstablished(
                        sessionTranscript: encodedSessionTranscript!.uint8Array(),
                        deviceRequest: deviceRequest.uint8Array()
                    )
                )
            }

            if let status {
                await finishSession(session, update: status.asCloseProximityDisclosureUpdate)
                return
            }
        }
    }

    private func finishSession(
        _ session: ActiveSession,
        update: CloseProximityDisclosureUpdate
    ) async {
        let shouldReport = clearActiveSessionIfCurrent(session)
        for transport in session.transports {
            try? await transport.close()
        }
        if shouldReport {
            try? await session.channel.sendUpdate(update: update)
        }
    }

    private func failSession(
        _ session: ActiveSession,
        error: Error
    ) async {
        let shouldReport = clearActiveSessionIfCurrent(session)
        for transport in session.transports {
            try? await transport.close()
        }
        if shouldReport {
            try? await session.channel.sendUpdate(
                update: CloseProximityDisclosureUpdate.error(
                    error: error.asCloseProximityDisclosureError)
            )
        }
    }
}

extension CloseProximityDisclosure: CloseProximityDisclosureBridge {
    func startQrHandover(channel: CloseProximityDisclosureChannel) async throws -> String {
        #if targetEnvironment(simulator)
            throw CloseProximityDisclosureError.PlatformError(
                reason: "Close proximity disclosure is not supported on the iOS Simulator"
            )
        #else
            try await lifecycleLock.withLock { [self] in
                await stopBleServerLocked()

                do {
                    let eDeviceKey = Crypto.shared.createEcPrivateKey(curve: .p256)
                    let connectionMethod = MdocConnectionMethodBle(
                        supportsPeripheralServerMode: true,
                        supportsCentralClientMode: false,
                        peripheralServerModeUuid: testingPeripheralServerModeUuid
                            ?? Multipaz.UUID.companion.randomUUID(),
                        centralClientModeUuid: nil,
                        peripheralServerModePsm: nil,
                        // iOS does not expose the local BLE MAC address to apps.
                        peripheralServerModeMacAddress: nil
                    )
                    let advertisedTransports = try await ConnectionHelperKt.advertise(
                        [connectionMethod],
                        role: .mdoc,
                        transportFactory: MdocTransportFactoryDefault(),
                        options: MdocTransportOptions(
                            bleUseL2CAP: false,
                            bleUseL2CAPInEngagement: false
                        )
                    )
                    let deviceEngagement = buildDeviceEngagement(
                        eDeviceKey: eDeviceKey.publicKey,
                        version: "1.0"
                    ) { builder in
                        advertisedTransports.forEach {
                            builder.addConnectionMethod(connectionMethod: $0.connectionMethod)
                        }
                    }
                    let encodedDeviceEngagement = Cbor.shared.encode(
                        item: deviceEngagement.toDataItem())
                    let qrCode = encodedDeviceEngagement.toBase64Url()
                    let connectionScope = MainScope()
                    let session = ActiveSession(
                        channel: channel,
                        transports: advertisedTransports,
                        connectionScope: connectionScope
                    )

                    setActiveSession(session)

                    let connectionTask = Task { [weak self] in
                        guard let self else { return }

                        do {
                            guard self.isActiveSession(session) else { return }

                            let transport = try await ConnectionHelperKt.waitForConnection(
                                advertisedTransports,
                                eSenderKey: eDeviceKey.publicKey,
                                coroutineScope: connectionScope
                            )
                            // A newer start/stop may already have replaced this session while the
                            // connection attempt was in flight. If that happened, just close the
                            // transport we obtained and exit without emitting more updates.
                            if !self.isActiveSession(session) {
                                try? await transport.close()
                                return
                            }
                            try await channel.sendUpdate(
                                update: CloseProximityDisclosureUpdate.connecting)

                            let transportStateObserverTask = Task { [weak self] in
                                guard let self else { return }
                                await self.observeTransportState(
                                    session: session, transport: transport)
                            }
                            session.setTransportStateObserverTask(transportStateObserverTask)

                            try await self.receiveMessages(
                                session: session,
                                transport: transport,
                                eDeviceKey: eDeviceKey,
                                encodedDeviceEngagement: encodedDeviceEngagement
                            )
                        } catch is CancellationError {
                            // No errors are needed on cancellation, as this is triggered from the cancelWaiter
                            // and core is assumed to clean up it's own state
                        } catch {
                            guard self.isActiveSession(session) else { return }
                            await self.failSession(session, error: error)
                        }
                    }
                    session.setConnectionTask(connectionTask)

                    return qrCode
                } catch {
                    try? await channel.sendUpdate(
                        update: CloseProximityDisclosureUpdate.error(
                            error: error.asCloseProximityDisclosureError)
                    )
                    throw error.asCloseProximityDisclosureError
                }
            }
        #endif
    }

    func sendDeviceResponse(deviceResponse: [UInt8]) async throws {
        // not implemented yet
    }

    func stopBleServer() async throws {
        try await lifecycleLock.withLock { [self] in
            await stopBleServerLocked()
        }
    }

    private func stopBleServerLocked() async {
        guard let session = takeActiveSession() else { return }

        // Take and clear the session first so any already-running background work immediately observes
        // itself as stale before we start canceling tasks and closing transports.
        await session.cancelBackgroundTasks()
        for transport in session.transports {
            try? await transport.close()
        }
        try? await session.channel.sendUpdate(update: CloseProximityDisclosureUpdate.closed)
    }
}

extension Error {
    fileprivate var asCloseProximityDisclosureError: CloseProximityDisclosureError {
        if let error = self as? CloseProximityDisclosureError {
            return error
        }
        return .from(self)
    }
}

extension KotlinByteArray {
    fileprivate var isEmpty: Bool {
        size == 0
    }

    fileprivate func uint8Array() -> [UInt8] {
        (0..<Int(size)).map { index in
            UInt8(bitPattern: get(index: Int32(index)))
        }
    }
}

extension NSNumber {
    fileprivate var asCloseProximityDisclosureUpdate: CloseProximityDisclosureUpdate {
        switch int64Value {
        case Constants.shared.SESSION_DATA_STATUS_SESSION_TERMINATION:
            return .closed
        case Constants.shared.SESSION_DATA_STATUS_ERROR_SESSION_ENCRYPTION:
            return .error(
                error: .PlatformError(
                    reason:
                        "Reader terminated the session with status 10 (session encryption error)"
                )
            )
        case Constants.shared.SESSION_DATA_STATUS_ERROR_CBOR_DECODING:
            return .error(
                error: .PlatformError(
                    reason: "Reader terminated the session with status 11 (CBOR decoding error)"
                )
            )
        default:
            return .error(
                error: .PlatformError(
                    reason: "Reader terminated the session with unexpected status \(int64Value)"
                )
            )
        }
    }
}
