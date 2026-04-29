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
        case advertising
        case connected
        case closed
        case failed
    }

    private enum Constants {
        static let startByte: UInt8 = 0x01
        static let endByte: UInt8 = 0x02
        static let pollIntervalNanoseconds: UInt64 = 10_000_000
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
    private var peripheralManager: CBPeripheralManager!
    private var service: CBMutableService?
    private var stateCharacteristic: CBMutableCharacteristic?
    private var clientToServerCharacteristic: CBMutableCharacteristic?
    private var serverToClientCharacteristic: CBMutableCharacteristic?
    private var state: State = .idle
    private var failure: Error?
    private var maximumCharacteristicSize = Constants.maxCharacteristicSize
    private var incomingMessageBuffer = Data()
    private var queuedMessages: [[UInt8]] = []

    init(serviceUuid: CBUUID) {
        self.serviceUuid = serviceUuid
        super.init()
        peripheralManager = CBPeripheralManager(delegate: self, queue: nil, options: nil)
    }

    func advertise() async throws {
        try expectState(.idle)
        try await waitForPoweredOn()
        try expectState(.idle)

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

        withLock {
            self.service = service
            self.stateCharacteristic = stateCharacteristic
            self.clientToServerCharacteristic = clientToServerCharacteristic
            self.serverToClientCharacteristic = serverToClientCharacteristic
            state = .advertising
        }

        log("Starting advertising for service \(serviceUuid.uuidString)")
        peripheralManager.add(service)
        peripheralManager.startAdvertising([
            CBAdvertisementDataServiceUUIDsKey: [serviceUuid],
        ])
    }

    func waitForConnection() async throws {
        try expectState(.advertising)

        while true {
            try Task.checkCancellation()

            let stateSnapshot = withLock { state }
            switch stateSnapshot {
            case .connected:
                return
            case .failed:
                throw currentFailure()
            case .closed:
                throw CloseProximityBleTransportError.transportClosed
            case .idle, .advertising:
                try await Task.sleep(nanoseconds: Constants.pollIntervalNanoseconds)
            }
        }
    }

    func waitForMessage() async throws -> [UInt8] {
        while true {
            try Task.checkCancellation()

            if let queuedMessage = withLock(body: popQueuedMessageLocked) {
                return queuedMessage
            }

            let stateSnapshot = withLock { state }
            switch stateSnapshot {
            case .failed:
                throw currentFailure()
            case .closed:
                throw CloseProximityBleTransportError.transportClosed
            case .idle, .advertising, .connected:
                try await Task.sleep(nanoseconds: Constants.pollIntervalNanoseconds)
            }
        }
    }

    func sendMessage(message: [UInt8]) async throws {
        try expectState(.connected)

        if message.isEmpty {
            try await writeStateByte(Constants.endByte)
            return
        }

        let maxChunkSize = max(maximumCharacteristicSize - 1, 1)
        log("sendMessage \(message.count) length")

        var offset = 0
        while offset < message.count {
            let remaining = message.count - offset
            let chunkSize = min(maxChunkSize, remaining)
            let moreDataComing = offset + chunkSize < message.count
            let chunk = [moreDataComing ? UInt8(0x01) : UInt8(0x00)] + Array(message[offset..<(offset + chunkSize)])
            try await writeNotificationChunk(chunk)
            offset += chunkSize
        }
        try await Task.sleep(nanoseconds: Constants.postSendSleepNanoseconds)

        log("sendMessage completed")
    }

    func close() throws {
        let shouldClose = withLock {
            switch state {
            case .closed:
                return false
            case .failed, .idle, .advertising, .connected:
                state = .closed
                queuedMessages.removeAll()
                incomingMessageBuffer.removeAll(keepingCapacity: false)
                return true
            }
        }

        guard shouldClose else { return }

        log("Closing BLE transport")
        peripheralManager.stopAdvertising()
        peripheralManager.removeAllServices()

        withLock {
            service = nil
            stateCharacteristic = nil
            clientToServerCharacteristic = nil
            serverToClientCharacteristic = nil
        }
    }

    private func waitForPoweredOn() async throws {
        while peripheralManager.state != .poweredOn {
            try Task.checkCancellation()

            let stateSnapshot = withLock { state }
            switch stateSnapshot {
            case .failed:
                throw currentFailure()
            case .closed:
                throw CloseProximityBleTransportError.transportClosed
            case .idle, .advertising, .connected:
                try await Task.sleep(nanoseconds: Constants.pollIntervalNanoseconds)
            }
        }
    }

    private func writeStateByte(_ value: UInt8) async throws {
        let chunk = [value]
        try await writeNotificationChunk(chunk, characteristic: stateCharacteristic)
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
        guard let prefix = chunk.first else {
            throw CloseProximityBleTransportError.invalidIncomingChunk(
                "Received empty client-to-server BLE chunk"
            )
        }

        incomingMessageBuffer.append(chunk.dropFirst())

        switch prefix {
        case 0x00:
            queuedMessages.append(Array(incomingMessageBuffer))
            incomingMessageBuffer.removeAll(keepingCapacity: false)
        case 0x01:
            if chunk.count != maximumCharacteristicSize {
                log("Received intermediate BLE chunk with unexpected size \(chunk.count), expected \(maximumCharacteristicSize)")
            }
        default:
            throw CloseProximityBleTransportError.invalidIncomingChunk(
                "Received BLE chunk with invalid prefix 0x\(String(prefix, radix: 16))"
            )
        }
    }

    private func enqueueTerminationMessage() {
        withLock {
            queuedMessages.append([])
        }
    }

    private func markConnected(maximumUpdateValueLength: Int) {
        let updatedMaximum = min(max(maximumUpdateValueLength, 1), Constants.maxCharacteristicSize)
        withLock {
            maximumCharacteristicSize = updatedMaximum
            state = .connected
        }
        log("BLE transport connected with max characteristic size \(updatedMaximum)")
        peripheralManager.stopAdvertising()
    }

    private func fail(_ error: Error) {
        let shouldFail = withLock {
            switch state {
            case .failed, .closed:
                return false
            case .idle, .advertising, .connected:
                failure = error
                state = .failed
                queuedMessages.removeAll()
                incomingMessageBuffer.removeAll(keepingCapacity: false)
                return true
            }
        }

        guard shouldFail else { return }

        log("BLE transport failed: \(error.localizedDescription)")
        peripheralManager.stopAdvertising()
        peripheralManager.removeAllServices()
    }

    private func expectState(_ expectedState: State) throws {
        let currentState = withLock { state }
        guard currentState == expectedState else {
            if currentState == .failed {
                throw currentFailure()
            }
            if currentState == .closed {
                throw CloseProximityBleTransportError.transportClosed
            }
            throw CloseProximityBleTransportError.invalidState(
                "Expected close proximity BLE transport state \(expectedState), got \(currentState)"
            )
        }
    }

    private func currentFailure() -> Error {
        withLock {
            failure
                ?? CloseProximityBleTransportError.transportFailed(
                    "Close proximity BLE transport failed without a specific error"
                )
        }
    }

    private func popQueuedMessageLocked() -> [UInt8]? {
        guard !queuedMessages.isEmpty else { return nil }
        return queuedMessages.removeFirst()
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
        guard peripheral.state != .poweredOn else { return }

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
                    try withLock {
                        try handleIncomingChunk(value)
                    }
                } catch {
                    fail(error)
                }
                continue
            }

            log("Ignoring unexpected write to characteristic \(request.characteristic.uuid.uuidString)")
        }
    }
}
