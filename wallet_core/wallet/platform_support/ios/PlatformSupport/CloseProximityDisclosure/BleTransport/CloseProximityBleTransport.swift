import CoreBluetooth
import Foundation

private enum CloseProximityBleTransportError: LocalizedError {
    case invalidState(String)
    case invalidIncomingChunk(String)
    case transportClosed
    case transportFailed(String)

    var errorDescription: String? {
        switch self {
        case .invalidState(let reason):
            return "InvalidState: \(reason)"
        case .invalidIncomingChunk(let reason):
            return "InvalidIncomingChunk: \(reason)"
        case .transportClosed:
            return "TransportClose: Close proximity BLE transport is closed"
        case .transportFailed(let reason):
            return "TransportFailed: \(reason)"
        }
    }
}

final class CloseProximityBleTransport: NSObject, @unchecked Sendable {
    enum State {
        case idle
        case startingAdvertising
        case advertising
        case connected
        case readerClosed
        case closed
        case failed
    }

    enum IncomingMessage {
        case payload([UInt8])
        case endOfStream
    }

    private enum WaitForMessageAction {
        case message(IncomingMessage)
        case wait
        case throwError(Error)
    }

    private enum WaitForAdvertisingAction {
        case started
        case wait
        case throwError(Error)
    }

    private enum MarkConnectedAction {
        case ignore
        case fail(Error)
        case connect(Int)
    }

    private struct AdvertisingStartup {
        var didAddService = false
        var didStartAdvertising = false
    }

    private enum Constants {
        static let minCharacteristicSize = 2
        static let finalChunkPrefix: UInt8 = 0x00
        static let continuationChunkPrefix: UInt8 = 0x01
        static let startByte: UInt8 = 0x01
        static let endByte: UInt8 = 0x02
        static let pollIntervalNanoseconds: UInt64 = 10_000_000
        static let advertisingStartupTimeoutNanoseconds: UInt64 = 2_000_000_000
        static let postSendSleepNanoseconds: UInt64 = 1_000_000_000
        static let maxCharacteristicSize = 512
    }

    static let stateCharacteristicUuid = CBUUID(string: "00000001-a123-48ce-896b-4c76973373e6")
    static let clientToServerCharacteristicUuid = CBUUID(string: "00000002-a123-48ce-896b-4c76973373e6")
    static let serverToClientCharacteristicUuid = CBUUID(string: "00000003-a123-48ce-896b-4c76973373e6")

    private let serviceUuid: CBUUID
    // Guards mutable transport state shared between CoreBluetooth delegate callbacks and async
    // wait/send/close methods on the transport.
    // Example race prevented: didReceiveWrite appends an incoming chunk while waitForMessage()
    // pops queued messages or close() clears the buffers, corrupting the transport state machine.
    private let lock = NSLock()
    // Serializes outbound sends across await points so concurrent callers cannot interleave
    // notification chunks and corrupt the BLE message framing.
    private let sendLock = CloseProximityDisclosureLifecycleLock()
    private var peripheralManager: CBPeripheralManager!
    private var service: CBMutableService?
    private var stateCharacteristic: CBMutableCharacteristic?
    private var clientToServerCharacteristic: CBMutableCharacteristic?
    private var serverToClientCharacteristic: CBMutableCharacteristic?
    private var state: State = .idle
    private var failure: Error?
    private var maximumCharacteristicSize = Constants.maxCharacteristicSize
    private var advertisingStartup: AdvertisingStartup?
    // Incoming message buffer, aggregating bytes before they are passed to the queuedMessages as full messages 
    private var incomingMessageBuffer = Data()
    // Hand-off buffer between CB delegate and the consumer
    private var queuedMessages: [IncomingMessage] = []

    init(serviceUuid: CBUUID) {
        self.serviceUuid = serviceUuid
        super.init()
        peripheralManager = CBPeripheralManager(delegate: self, queue: nil, options: nil)
    }

    func advertise() async throws {
        try await waitForPoweredOnAndIdle()

        let stateCharacteristic = CBMutableCharacteristic(
            type: Self.stateCharacteristicUuid,
            properties: [.notify, .writeWithoutResponse],
            value: nil,
            permissions: [.writeable]
        )
        let clientToServerCharacteristic = CBMutableCharacteristic(
            type: Self.clientToServerCharacteristicUuid,
            properties: [.writeWithoutResponse],
            value: nil,
            permissions: [.writeable]
        )
        let serverToClientCharacteristic = CBMutableCharacteristic(
            type: Self.serverToClientCharacteristicUuid,
            properties: [.notify],
            value: nil,
            permissions: [.readable, .writeable]
        )

        let service = CBMutableService(type: serviceUuid, primary: true)
        service.characteristics = [
            stateCharacteristic,
            clientToServerCharacteristic,
            serverToClientCharacteristic,
        ]

        try withLock {
            guard state == .idle else {
               throw CloseProximityBleTransportError.invalidState(
                    "Expected close proximity BLE transport state .idle on advertise, got \(state)"
                )
            }
            self.service = service
            self.stateCharacteristic = stateCharacteristic
            self.clientToServerCharacteristic = clientToServerCharacteristic
            self.serverToClientCharacteristic = serverToClientCharacteristic
            advertisingStartup = AdvertisingStartup()
            state = .startingAdvertising
        }

        log("Starting advertising for service \(serviceUuid.uuidString)")
        // This is outside of the lock to prevent re-entrancy issues on callbacks.
        // It's impossible for 2 threads to reach this code because the 2nd thread
        // would fail the state check in the lock above.
        peripheralManager.add(service)
        peripheralManager.startAdvertising([
            CBAdvertisementDataServiceUUIDsKey: [serviceUuid],
        ])
        let startupBeganAt = DispatchTime.now().uptimeNanoseconds

        while true {
            try Task.checkCancellation()

            switch withLock(body: nextWaitForAdvertisingActionLocked) {
            case .started:
                return
            case .throwError(let error):
                throw error
            case .wait:
                if DispatchTime.now().uptimeNanoseconds - startupBeganAt
                    >= Constants.advertisingStartupTimeoutNanoseconds
                {
                    let error = CloseProximityBleTransportError.transportFailed(
                        "Timed out waiting for BLE advertising startup callbacks"
                    )
                    fail(error)
                    throw error
                }
                try await Task.sleep(nanoseconds: Constants.pollIntervalNanoseconds)
            }
        }
    }

    func waitForConnection() async throws {

        while true {
            try Task.checkCancellation()

            let stateSnapshot = getStateSnapshot()
            switch stateSnapshot {
            case .connected:
                return
            case .failed:
                throw currentFailure()
            case .readerClosed, .closed:
                throw CloseProximityBleTransportError.transportClosed
            case .idle, .startingAdvertising, .advertising:
                try await Task.sleep(nanoseconds: Constants.pollIntervalNanoseconds)
            }
        }
    }

    func waitForMessage() async throws -> IncomingMessage {
        while true {
            try Task.checkCancellation()

            switch withLock(body: nextWaitForMessageActionLocked) {
            case .message(let queuedMessage):
                return queuedMessage
            case .throwError(let error):
                throw error
            case .wait:
                try await Task.sleep(nanoseconds: Constants.pollIntervalNanoseconds)
            }
        }
    }

    func sendMessage(message: [UInt8]) async throws {
        try await sendLock.withLock { [self] in
            try expectState(.connected)

            if message.isEmpty {
                try await writeStateByte(Constants.endByte)
                return
            }

            let maxChunkSize = withLock { maximumCharacteristicSize - 1 }
            log("sendMessage \(message.count) length")

            var offset = 0
            while offset < message.count {
                let remaining = message.count - offset
                let chunkSize = min(maxChunkSize, remaining)
                let moreDataComing = offset + chunkSize < message.count
                let chunk = [
                    moreDataComing
                        ? Constants.continuationChunkPrefix
                        : Constants.finalChunkPrefix,
                ] + Array(message[offset..<(offset + chunkSize)])
                try await writeNotificationChunk(chunk)
                offset += chunkSize
            }
            try await Task.sleep(nanoseconds: Constants.postSendSleepNanoseconds)

            log("sendMessage completed")
        }
    }

    func close() throws {
        let shouldClose = withLock {
            switch state {
            case .closed:
                return false
            case .failed, .idle, .startingAdvertising, .advertising, .connected, .readerClosed:
                state = .closed
                advertisingStartup = nil
                queuedMessages.removeAll()
                incomingMessageBuffer.removeAll(keepingCapacity: false)
                return true
            }
        }

        guard shouldClose else { return }

        log("Closing BLE transport")
        // This is a confirmed no-op when called multiple times
        peripheralManager.stopAdvertising()
        peripheralManager.removeAllServices()

        withLock {
            service = nil
            stateCharacteristic = nil
            clientToServerCharacteristic = nil
            serverToClientCharacteristic = nil
        }
    }

    private func waitForPoweredOnAndIdle() async throws {
        while peripheralManager.state != .poweredOn {
            try Task.checkCancellation()
            try expectState(.idle)
            try await Task.sleep(nanoseconds: Constants.pollIntervalNanoseconds)
        }

        try expectState(.idle)
    }

    private func writeStateByte(_ value: UInt8) async throws {
        let chunk = [value]
        let characteristic = withLock { stateCharacteristic }
        try await writeNotificationChunk(chunk, characteristic: characteristic)
    }

    private func writeNotificationChunk(
        _ chunk: [UInt8],
        characteristic explicitCharacteristic: CBMutableCharacteristic? = nil
    ) async throws {
        let characteristic = withLock {
            explicitCharacteristic ?? serverToClientCharacteristic
        }
        guard let characteristic else {
            throw CloseProximityBleTransportError.invalidState(
                "Close proximity BLE transport has no writable characteristic"
            )
        }

        while true {
            try Task.checkCancellation()
            try expectState(.connected)

            let wasSent = peripheralManager.updateValue(
                Data(chunk),
                for: characteristic,
                onSubscribedCentrals: nil
            )

            if wasSent {
                log("Wrote \(chunk.count) bytes to characteristic \(characteristic.uuid.uuidString)")
                return
            }

            log("Not ready to send to characteristic \(characteristic.uuid.uuidString), retrying after a short delay")
            try await Task.sleep(nanoseconds: Constants.pollIntervalNanoseconds)
        }
    }

    private func handleIncomingChunk(_ chunk: Data) throws {
        let didHandleChunk = try withLock {
            switch state {
            case .connected:
                break
            case .readerClosed:
                throw CloseProximityBleTransportError.invalidIncomingChunk(
                    "Received client-to-server BLE chunk after reader termination"
                )
            case .idle, .startingAdvertising, .advertising, .closed, .failed:
                return false
            }

            guard let prefix = chunk.first else {
                throw CloseProximityBleTransportError.invalidIncomingChunk(
                    "Received empty client-to-server BLE chunk"
                )
            }

            incomingMessageBuffer.append(chunk.dropFirst())

            switch prefix {
            case Constants.finalChunkPrefix:
                queuedMessages.append(.payload(Array(incomingMessageBuffer)))
                incomingMessageBuffer.removeAll(keepingCapacity: false)
            case Constants.continuationChunkPrefix:
                if chunk.count != maximumCharacteristicSize {
                    log("Received intermediate BLE chunk with unexpected size \(chunk.count), expected \(maximumCharacteristicSize)")
                }
            default:
                throw CloseProximityBleTransportError.invalidIncomingChunk(
                    "Received BLE chunk with invalid prefix 0x\(String(prefix, radix: 16))"
                )
            }

            return true
        }

        guard didHandleChunk else {
            log("Ignoring client-to-server BLE chunk while not connected")
            return
        }
    }

    private func enqueueTerminationMessage() {
        let didEnqueue = withLock {
            guard state == .connected else { return false }
            state = .readerClosed
            queuedMessages.append(.endOfStream)
            return true
        }

        guard didEnqueue else {
            log("Ignoring BLE transport termination byte while not connected")
            return
        }
    }

    private func markConnected(maximumUpdateValueLength: Int) {
        let action: MarkConnectedAction = withLock {
            guard state == .advertising else { return .ignore }
            guard maximumUpdateValueLength >= Constants.minCharacteristicSize else {
                return .fail(CloseProximityBleTransportError.transportFailed(
                    "Connected central reported maximumUpdateValueLength \(maximumUpdateValueLength), expected at least \(Constants.minCharacteristicSize)"
                ))
            }

            let updatedMaximum = min(maximumUpdateValueLength, Constants.maxCharacteristicSize)
            maximumCharacteristicSize = updatedMaximum
            state = .connected
            return .connect(updatedMaximum)
        }

        switch action {
        case .ignore:
            log("Ignoring BLE transport start byte while not advertising")
            return
        case .fail(let error):
            fail(error)
            return
        case .connect(let updatedMaximum):
            log("BLE transport connected with max characteristic size \(updatedMaximum)")
            peripheralManager.stopAdvertising()
        }
    }

    private func fail(_ error: Error) {
        let shouldFail = withLock {
            switch state {
            case .failed, .closed:
                return false
            case .idle, .startingAdvertising, .advertising, .connected, .readerClosed:
                failure = error
                advertisingStartup = nil
                state = .failed
                queuedMessages.removeAll()
                incomingMessageBuffer.removeAll(keepingCapacity: false)
                return true
            }
        }

        guard shouldFail else { return }

        log("BLE transport failed: \(error.localizedDescription)")
        // This is a confirmed no-op when called multiple times
        peripheralManager.stopAdvertising()
        peripheralManager.removeAllServices()
    }

    private func expectState(_ expectedState: State) throws {
        let currentState = getStateSnapshot()
        guard currentState == expectedState else {
            if currentState == .failed {
                throw currentFailure()
            }
            if currentState == .readerClosed || currentState == .closed {
                throw CloseProximityBleTransportError.transportClosed
            }
            throw CloseProximityBleTransportError.invalidState(
                "Expected close proximity BLE transport state \(expectedState), got \(currentState)"
            )
        }
    }

    private func getStateSnapshot() -> CloseProximityBleTransport.State {
        return withLock { state }
    }

    private func currentFailure() -> Error {
        withLock(body: currentFailureLocked)
    }

    private func currentFailureLocked() -> Error {
        failure
            ?? CloseProximityBleTransportError.transportFailed(
                "Close proximity BLE transport failed without a specific error"
            )
    }

    private func popQueuedMessageLocked() -> IncomingMessage? {
        guard !queuedMessages.isEmpty else { return nil }
        return queuedMessages.removeFirst()
    }

    private func nextWaitForMessageActionLocked() -> WaitForMessageAction {
        switch state {
        case .failed:
            return .throwError(currentFailureLocked())
        case .closed:
            return .throwError(CloseProximityBleTransportError.transportClosed)
        case .readerClosed:
            if let queuedMessage = popQueuedMessageLocked() {
                return .message(queuedMessage)
            }
            return .throwError(CloseProximityBleTransportError.transportClosed)
        case .idle, .startingAdvertising, .advertising, .connected:
            if let queuedMessage = popQueuedMessageLocked() {
                return .message(queuedMessage)
            }
            return .wait
        }
    }

    private func nextWaitForAdvertisingActionLocked() -> WaitForAdvertisingAction {
        switch state {
        case .advertising, .connected:
            return .started
        case .startingAdvertising:
            return .wait
        case .failed:
            return .throwError(currentFailureLocked())
        case .readerClosed, .closed:
            return .throwError(CloseProximityBleTransportError.transportClosed)
        case .idle:
            return .throwError(
                CloseProximityBleTransportError.invalidState(
                    "Advertising startup unexpectedly returned to .idle"
                )
            )
        }
    }

    private func didAddService(_ service: CBService, error: Error?) {
        if let error {
            fail(
                CloseProximityBleTransportError.transportFailed(
                    "Failed to add BLE service \(service.uuid.uuidString): \(error.localizedDescription)"
                )
            )
            return
        }

        advanceAdvertisingStartup(markServiceAdded: true)
    }

    private func didStartAdvertising(error: Error?) {
        if let error {
            fail(
                CloseProximityBleTransportError.transportFailed(
                    "Failed to start BLE advertising for service \(serviceUuid.uuidString): \(error.localizedDescription)"
                )
            )
            return
        }

        advanceAdvertisingStartup(markAdvertisingStarted: true)
    }

    private func advanceAdvertisingStartup(
        markServiceAdded: Bool = false,
        markAdvertisingStarted: Bool = false
    ) {
        let didFinishStarting = withLock {
            guard state == .startingAdvertising, var advertisingStartup else { return false }

            if markServiceAdded {
                advertisingStartup.didAddService = true
            }
            if markAdvertisingStarted {
                advertisingStartup.didStartAdvertising = true
            }

            guard advertisingStartup.didAddService && advertisingStartup.didStartAdvertising else {
                self.advertisingStartup = advertisingStartup
                return false
            }

            self.advertisingStartup = nil
            state = .advertising
            return true
        }

        if didFinishStarting {
            log("Advertising started for service \(serviceUuid.uuidString)")
        }
    }

    private func withLock<T>(body: () throws -> T) rethrows -> T {
        lock.lock()
        defer { lock.unlock() }
        return try body()
    }

    private func log(_ message: String) {
        #if DEBUG
        NSLog("[CloseProximityBleTransport] %@", message)
        #endif
    }
}

extension CloseProximityBleTransport: CBPeripheralManagerDelegate {
    func peripheralManagerDidUpdateState(_ peripheral: CBPeripheralManager) {
        switch peripheral.state {
        case .poweredOn:
            return
        case .resetting, .unauthorized, .unsupported, .poweredOff:
            fail(
                CloseProximityBleTransportError.transportFailed(
                    "CBPeripheralManager is not powered on: \(peripheral.state.rawValue)"
                )
            )
        case .unknown:
            return
        @unknown default:
            return
        }
    }

    func peripheralManager(
        _ peripheral: CBPeripheralManager,
        didReceiveWrite requests: [CBATTRequest]
    ) {
        for request in requests {
            let value = request.value ?? Data()

            if request.characteristic.uuid == Self.stateCharacteristicUuid {
                if value == Data([Constants.startByte]) {
                    markConnected(maximumUpdateValueLength: request.central.maximumUpdateValueLength)
                } else if value == Data([Constants.endByte]) {
                    log("Received BLE transport termination byte from reader")
                    enqueueTerminationMessage()
                } else {
                    log("Ignoring unexpected state characteristic write: \(value as NSData)")
                }
                continue
            }

            if request.characteristic.uuid == Self.clientToServerCharacteristicUuid {
                do {
                    try handleIncomingChunk(value)
                } catch {
                    fail(error)
                }
                continue
            }

            log("Ignoring unexpected write to characteristic \(request.characteristic.uuid.uuidString)")
        }
    }

    func peripheralManager(
        _ peripheral: CBPeripheralManager,
        didAdd service: CBService,
        error: Error?
    ) {
        didAddService(service, error: error)
    }

    func peripheralManagerDidStartAdvertising(
        _ peripheral: CBPeripheralManager,
        error: Error?
    ) {
        didStartAdvertising(error: error)
    }
}
