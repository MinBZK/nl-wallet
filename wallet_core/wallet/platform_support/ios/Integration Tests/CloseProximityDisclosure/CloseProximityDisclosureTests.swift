//
//  CloseProximityDisclosureTests.swift
//  Integration Tests
//
//  Created by The Wallet Developers on 06/03/2026.
//

import CoreBluetooth
import CryptoKit
import Foundation
@preconcurrency import Multipaz
import XCTest

@testable import PlatformSupport

final class CloseProximityDisclosureTests: XCTestCase {
    private enum RawCleanupAction: String, Sendable {
        case stopAdvertising = "stopAdvertising"
        case removeAllServices = "removeAllServices"
    }

    private final class RawPeripheralManagerCharacterizer: NSObject, CBPeripheralManagerDelegate {
        private actor State {
            private var isPoweredOn = false
            private var events: [String] = []

            func recordState(_ state: CBManagerState) {
                isPoweredOn = state == .poweredOn
                events.append("state=\(state.summary)")
            }

            func recordEvent(_ event: String) {
                events.append(event)
            }

            func poweredOn() -> Bool {
                isPoweredOn
            }

            func eventCount() -> Int {
                events.count
            }

            func snapshot() -> [String] {
                events
            }
        }

        private enum Constants {
            static let characteristicUuid = CBUUID(
                string: "00000010-a123-48ce-896b-4c76973373e6"
            )
        }

        private let state = State()
        private(set) var peripheralManager: CBPeripheralManager!

        override init() {
            super.init()
            peripheralManager = CBPeripheralManager(delegate: self, queue: nil, options: nil)
        }

        func makeService(serviceUuid: CBUUID) -> CBMutableService {
            let characteristic = CBMutableCharacteristic(
                type: Constants.characteristicUuid,
                properties: [.read, .writeWithoutResponse],
                value: nil,
                permissions: [.readable, .writeable]
            )

            let service = CBMutableService(type: serviceUuid, primary: true)
            service.characteristics = [characteristic]
            return service
        }

        func isPoweredOn() async -> Bool {
            await state.poweredOn()
        }

        func eventCount() async -> Int {
            await state.eventCount()
        }

        func events() async -> [String] {
            await state.snapshot()
        }

        func stop() {
            peripheralManager.stopAdvertising()
            peripheralManager.removeAllServices()
        }

        func peripheralManagerDidUpdateState(_ peripheral: CBPeripheralManager) {
            Task {
                await state.recordState(peripheral.state)
            }
        }

        func peripheralManager(
            _ peripheral: CBPeripheralManager,
            didAdd service: CBService,
            error: Error?
        ) {
            Task {
                await state.recordEvent(
                    "didAdd service=\(service.uuid.uuidString) error=\(error?.localizedDescription ?? "nil")"
                )
            }
        }

        func peripheralManagerDidStartAdvertising(
            _ peripheral: CBPeripheralManager,
            error: Error?
        ) {
            Task {
                await state.recordEvent(
                    "didStartAdvertising error=\(error?.localizedDescription ?? "nil") isAdvertising=\(peripheral.isAdvertising)"
                )
            }
        }
    }

    private enum AdvertiseAttemptResult: Equatable, Sendable {
        case started
        case failed(String)

        var summary: String {
            switch self {
            case .started:
                return "started"
            case .failed(let message):
                return "failed(\(message))"
            }
        }
    }

    private enum RecordedUpdate: Equatable, Sendable {
        case connected
        case sessionEstablished(sessionTranscript: [UInt8], deviceRequest: [UInt8])
        case closed
        case other
    }

    private final class TestChannel: CloseProximityDisclosureChannel, @unchecked Sendable {
        private actor State {
            private var updates: [RecordedUpdate] = []

            func record(update: CloseProximityDisclosureUpdate) {
                switch update {
                case .connected:
                    updates.append(.connected)
                case .sessionEstablished(let sessionTranscript, let deviceRequest):
                    updates.append(
                        .sessionEstablished(
                            sessionTranscript: sessionTranscript,
                            deviceRequest: deviceRequest
                        )
                    )
                case .closed:
                    updates.append(.closed)
                default:
                    updates.append(.other)
                }
            }

            func hasReceivedConnectedUpdate() -> Bool {
                updates.contains(.connected)
            }

            func hasReceivedClosedUpdate() -> Bool {
                updates.contains(.closed)
            }

            func hasReceivedSessionEstablishedUpdate() -> Bool {
                updates.contains(where: { update in
                    if case .sessionEstablished = update {
                        return true
                    }
                    return false
                })
            }

            func receivedSessionEstablishedUpdate() -> (
                sessionTranscript: [UInt8],
                deviceRequest: [UInt8]
            )? {
                for update in updates {
                    if case .sessionEstablished(let sessionTranscript, let deviceRequest) = update {
                        return (sessionTranscript, deviceRequest)
                    }
                }

                return nil
            }

            func receivedUpdates() -> [RecordedUpdate] {
                updates
            }
        }

        private let state = State()

        init() {
            super.init(noHandle: NoHandle())
        }

        required init(unsafeFromHandle handle: UInt64) {
            super.init(unsafeFromHandle: handle)
        }

        override func sendUpdate(update: CloseProximityDisclosureUpdate) async throws {
            await state.record(update: update)
        }

        func hasReceivedConnectedUpdate() async -> Bool {
            await state.hasReceivedConnectedUpdate()
        }

        func hasReceivedClosedUpdate() async -> Bool {
            await state.hasReceivedClosedUpdate()
        }

        func hasReceivedSessionEstablishedUpdate() async -> Bool {
            await state.hasReceivedSessionEstablishedUpdate()
        }

        func receivedSessionEstablishedUpdate() async -> (
            sessionTranscript: [UInt8],
            deviceRequest: [UInt8]
        )? {
            await state.receivedSessionEstablishedUpdate()
        }

        func receivedUpdates() async -> [RecordedUpdate] {
            await state.receivedUpdates()
        }
    }

    private actor StartGate {
        private let parties: Int
        private var arrived = 0
        private var continuations: [CheckedContinuation<Void, Never>] = []

        init(parties: Int) {
            self.parties = parties
        }

        func wait() async {
            arrived += 1

            guard arrived < parties else {
                let continuations = continuations
                self.continuations.removeAll()
                continuations.forEach { $0.resume() }
                return
            }

            await withCheckedContinuation { continuation in
                continuations.append(continuation)
            }
        }
    }

    static var platformSupport: PlatformSupport?
    private static let runMacBleReaderFullFlowTest = true
    private static let macBleReaderMarker = "CLOSE_PROXIMITY_MAC_READER="
    private static let macBleReaderDeterministicReaderPrivateKeyHex =
        "de3b4b9e5f72dd9b58406ae3091434da48a6f9fd010d88fcb0958e2cebec947c"
    private static let macBleReaderExpectedDeviceRequest: [UInt8] = [0x01, 0x02, 0x03]
    private static let macBleReaderExpectedDeviceResponse: [UInt8] = [0x04, 0x05, 0x06]
    private static let macBleReaderExpectedDeviceResponseHex = "040506"

    override class func setUp() {
        self.platformSupport = PlatformSupport.shared
    }

    func testAllCloseProximityDisclosures() async throws {
        // The Rust code will panic if this test fails.
        await Task.detached {
            close_proximity_disclosure_test_start_qr_handover()
        }.value
    }

    func testStartQrHandoverStartsAndStopsBleServer() async throws {
        let closeProximityDisclosure = CloseProximityDisclosure()
        let channel = TestChannel()

        let isBleServerActiveBeforeStart = await closeProximityDisclosure.isBleServerActiveForTesting()
        XCTAssertFalse(isBleServerActiveBeforeStart)

        let qrCode = try await closeProximityDisclosure.startQrHandover(channel: channel)

        NSLog("Close proximity disclosure QR code: %@", qrCode)

        XCTAssertFalse(qrCode.isEmpty)
        XCTAssertFalse(qrCode.hasPrefix("mdoc:"))
        let isBleServerActiveAfterStart = await closeProximityDisclosure.isBleServerActiveForTesting()
        XCTAssertTrue(isBleServerActiveAfterStart)

        try await closeProximityDisclosure.stopBleServer()

        let hasReceivedClosedUpdate = await channel.hasReceivedClosedUpdate()
        XCTAssertTrue(hasReceivedClosedUpdate)
        let isBleServerActiveAfterStop = await closeProximityDisclosure.isBleServerActiveForTesting()
        XCTAssertFalse(isBleServerActiveAfterStop)
    }

    func testConcurrentAdvertiseCallsOnSameTransportRejectSecondCaller() async throws {
        #if targetEnvironment(simulator)
            throw XCTSkip("CBPeripheralManager peripheral mode is not supported on the iOS Simulator")
        #endif

        let transport = CloseProximityBleTransport(
            serviceUuid: CBUUID(string: UUID().uuidString)
        )
        let startGate = StartGate(parties: 2)

        // Characterize the race around peripheralManager.add/startAdvertising. Only one caller
        // should flip the transport from .idle to .advertising; the other must fail before it can
        // issue a second CoreBluetooth advertise sequence on the same transport.
        async let firstAttempt = advertiseAfterGate(transport: transport, startGate: startGate)
        async let secondAttempt = advertiseAfterGate(transport: transport, startGate: startGate)

        let firstResult = await firstAttempt
        let secondResult = await secondAttempt
        let results = [firstResult, secondResult]
        let resultSummary = results.map(\.summary).joined(separator: ", ")

        NSLog("Concurrent advertise() results: %@", resultSummary)

        let startedCount = results.filter { result in
            if case .started = result {
                return true
            }
            return false
        }.count
        let failureMessages = results.compactMap { result in
            if case .failed(let message) = result {
                return message
            }
            return nil
        }

        XCTAssertEqual(
            startedCount,
            1,
            "Expected exactly one advertise() call to start advertising, got \(resultSummary)"
        )
        XCTAssertEqual(
            failureMessages.count,
            1,
            "Expected exactly one advertise() failure, got \(resultSummary)"
        )
        XCTAssertTrue(
            failureMessages.first?.contains(
                "Expected close proximity BLE transport state .idle on advertise, got startingAdvertising"
            ) == true
                || failureMessages.first?.contains(
                    "Expected close proximity BLE transport state .idle on advertise, got advertising"
                ) == true,
            "Expected the losing caller to be rejected because the transport is already starting or already advertising, got \(resultSummary)"
        )

        try transport.close()
    }

    @MainActor
    func testRawPeripheralManagerRepeatedAddAndStartAdvertisingCharacterization() async throws {
        #if targetEnvironment(simulator)
            throw XCTSkip("CBPeripheralManager peripheral mode is not supported on the iOS Simulator")
        #endif

        let characterizer = RawPeripheralManagerCharacterizer()
        let didPowerOn = await waitUntil(timeoutNanoseconds: 5_000_000_000) {
            await characterizer.isPoweredOn()
        }
        let startupEvents = await characterizer.events()

        XCTAssertTrue(
            didPowerOn,
            "Expected CBPeripheralManager to become powered on, got \(startupEvents)"
        )

        let serviceUuid = CBUUID(string: UUID().uuidString)
        let firstService = characterizer.makeService(serviceUuid: serviceUuid)
        let secondService = characterizer.makeService(serviceUuid: serviceUuid)

        NSLog(
            "Raw CBPeripheralManager repeated add/start begin service=%@",
            serviceUuid.uuidString
        )

        characterizer.peripheralManager.add(firstService)
        characterizer.peripheralManager.startAdvertising([
            CBAdvertisementDataServiceUUIDsKey: [serviceUuid],
        ])
        let isAdvertisingAfterFirstStart = characterizer.peripheralManager.isAdvertising

        characterizer.peripheralManager.add(secondService)
        characterizer.peripheralManager.startAdvertising([
            CBAdvertisementDataServiceUUIDsKey: [serviceUuid],
        ])
        let isAdvertisingAfterSecondStart = characterizer.peripheralManager.isAdvertising

        NSLog(
            "Raw CBPeripheralManager immediate isAdvertising after first start call: %@",
            String(isAdvertisingAfterFirstStart)
        )
        NSLog(
            "Raw CBPeripheralManager immediate isAdvertising after second start call: %@",
            String(isAdvertisingAfterSecondStart)
        )

        _ = await waitUntil(timeoutNanoseconds: 2_000_000_000) {
            await characterizer.eventCount() >= 4
        }

        let events = await characterizer.events()
        for event in events {
            NSLog("Raw CBPeripheralManager event: %@", event)
        }

        let didAddCount = events.filter { $0.contains("didAdd") }.count
        let didStartAdvertisingCount = events.filter { $0.contains("didStartAdvertising") }.count

        XCTAssertGreaterThanOrEqual(
            didAddCount,
            1,
            "Expected at least one didAdd callback, got \(events)"
        )
        XCTAssertGreaterThanOrEqual(
            didStartAdvertisingCount,
            1,
            "Expected at least one didStartAdvertising callback, got \(events)"
        )

        characterizer.stop()
    }

    @MainActor
    func testRawPeripheralManagerStopThenRemoveAllServicesCharacterization() async throws {
        try await runRawPeripheralManagerCleanupPermutationTest(
            name: "stop->remove",
            actions: [.stopAdvertising, .removeAllServices]
        )
    }

    @MainActor
    func testRawPeripheralManagerRemoveAllServicesThenStopAdvertisingCharacterization() async throws {
        try await runRawPeripheralManagerCleanupPermutationTest(
            name: "remove->stop",
            actions: [.removeAllServices, .stopAdvertising]
        )
    }

    @MainActor
    func testRawPeripheralManagerRepeatedStopAndRemoveCharacterization() async throws {
        try await runRawPeripheralManagerCleanupPermutationTest(
            name: "stop->stop->remove->remove",
            actions: [
                .stopAdvertising,
                .stopAdvertising,
                .removeAllServices,
                .removeAllServices,
            ]
        )
    }

    @MainActor
    func testRawPeripheralManagerStopAndRemoveWhileIdleCharacterization() async throws {
        #if targetEnvironment(simulator)
            throw XCTSkip("CBPeripheralManager peripheral mode is not supported on the iOS Simulator")
        #endif

        let characterizer = RawPeripheralManagerCharacterizer()
        try await waitForRawPeripheralManagerPoweredOn(characterizer)

        NSLog(
            "Raw idle cleanup begin isAdvertising=%@",
            String(characterizer.peripheralManager.isAdvertising)
        )

        characterizer.peripheralManager.stopAdvertising()
        NSLog(
            "Raw idle cleanup immediate isAdvertising after stopAdvertising: %@",
            String(characterizer.peripheralManager.isAdvertising)
        )

        characterizer.peripheralManager.removeAllServices()
        NSLog(
            "Raw idle cleanup immediate isAdvertising after removeAllServices: %@",
            String(characterizer.peripheralManager.isAdvertising)
        )

        try await Task.sleep(nanoseconds: 200_000_000)
        NSLog(
            "Raw idle cleanup isAdvertising after 200ms: %@",
            String(characterizer.peripheralManager.isAdvertising)
        )

        let restartEvents = await startRawPeripheralManagerAdvertising(
            characterizer: characterizer,
            label: "idle cleanup restart"
        )

        XCTAssertTrue(
            restartEvents.contains(where: { $0.contains("didStartAdvertising error=nil") }),
            "Expected advertising to start cleanly after idle stop/remove cleanup, got \(restartEvents)"
        )

        characterizer.stop()
    }

    func testKotlinByteArrayBase64UrlEncodedStringUsesUnsignedBytesAndOmitsPadding() {
        let bytes: [UInt8] = [0xFB, 0xFF, 0x00]

        XCTAssertEqual(bytes.kotlinByteArray().base64UrlEncodedString(), "-_8A")
    }

    func testGetEReaderKeyWithMissingReaderKeyLogsWhetherMultipazThrowsOrAborts() throws {
        // choosing to throw here instead of removing them from each scheme, to prevent accidentally turning them on on new schemes/targets
        throw XCTSkip("this test can be used to check multipaz methods failure handling, it will crash on current multipaz version")
        // Valid CBOR for {"data": h''}; the payload is structurally a session-establishment
        // message, but it omits the required eReaderKey field.
        let malformedSessionEstablishmentMessage: [UInt8] = [
            0xA1,
            0x64, 0x64, 0x61, 0x74, 0x61,
            0x40,
        ]

        let payloadHex = malformedSessionEstablishmentMessage
            .map { String(format: "%02x", $0) }
            .joined()

        _ = SessionEncryption.companion.getEReaderKey(
            sessionEstablishmentMessage: malformedSessionEstablishmentMessage.kotlinByteArray()
        )
    }

    func testDecryptMessageWithBogusCiphertextLogsWhetherMultipazThrowsOrAborts() throws {
        // choosing to throw here instead of removing them from each scheme, to prevent accidentally turning them on on new schemes/targets
        throw XCTSkip("this test can be used to check multipaz methods failure handling, it will crash on current multipaz version")
        let sessionEncryption = makeSessionEncryptionForRawDecryptCharacterization()
        // Valid CBOR for {"data": h'00010203'}; the payload has the expected SessionData shape,
        // but the ciphertext is intentionally bogus so decryptMessage has to decide how it fails.
        let malformedSessionDataMessage: [UInt8] = [
            0xA1,
            0x64, 0x64, 0x61, 0x74, 0x61,
            0x44, 0x00, 0x01, 0x02, 0x03,
        ]

        let payloadHex = malformedSessionDataMessage
            .map { String(format: "%02x", $0) }
            .joined()

        let decryptedMessage = sessionEncryption.decryptMessage(
            messageData: malformedSessionDataMessage.kotlinByteArray()
        )

    }

    func testCloseProximityDisclosureFullFlowWithMacReader() async throws {
        guard Self.runMacBleReaderFullFlowTest else {
            throw XCTSkip(
                """
                Set runMacBleReaderFullFlowTest = true and run \
                scripts/close_proximity/disclosure_mac_reader.swift --qr-code <logged-qr-code> \
                --expect-device-response-hex \(Self.macBleReaderExpectedDeviceResponseHex) \
                on the host Mac to exercise the full flow.
                """
            )
        }

        let closeProximityDisclosure = CloseProximityDisclosure()
        let channel = TestChannel()

        let isBleServerActiveBeforeStart = await closeProximityDisclosure.isBleServerActiveForTesting()
        XCTAssertFalse(isBleServerActiveBeforeStart)

        let qrCode = try await closeProximityDisclosure.startQrHandover(channel: channel)

        NSLog("Close proximity disclosure QR code: %@", qrCode)
        NSLog(
            "%@",
            Self.macBleReaderMarkerPayload(
                qrCode: qrCode,
                expectedDeviceResponseHex: Self.macBleReaderExpectedDeviceResponseHex
            )
        )

        XCTAssertFalse(qrCode.isEmpty)
        XCTAssertFalse(qrCode.hasPrefix("mdoc:"))
        let isBleServerActiveAfterStart = await closeProximityDisclosure.isBleServerActiveForTesting()
        XCTAssertTrue(isBleServerActiveAfterStart)

        let didReceiveSessionEstablished = await waitUntil(timeoutNanoseconds: 30_000_000_000) {
            await channel.hasReceivedSessionEstablishedUpdate()
        }
        XCTAssertTrue(
            didReceiveSessionEstablished,
            """
            Timed out waiting for the host Mac BLE helper to send SessionEstablished. \
            Run scripts/close_proximity/disclosure_mac_reader.swift --qr-code <logged-qr-code> \
            --expect-device-response-hex \(Self.macBleReaderExpectedDeviceResponseHex) \
            with the QR code logged by this test.
            """
        )

        guard let sessionEstablished = await channel.receivedSessionEstablishedUpdate() else {
            XCTFail("Expected a SessionEstablished update")
            return
        }

        XCTAssertEqual(sessionEstablished.deviceRequest, Self.macBleReaderExpectedDeviceRequest)
        XCTAssertEqual(
            sessionEstablished.sessionTranscript,
            try expectedSessionTranscript(forQrCode: qrCode)
        )

        try await closeProximityDisclosure.sendDeviceResponse(
            deviceResponse: Self.macBleReaderExpectedDeviceResponse
        )

        let didReceiveClosed = await waitUntil(timeoutNanoseconds: 5_000_000_000) {
            await channel.hasReceivedClosedUpdate()
        }
        XCTAssertTrue(
            didReceiveClosed,
            """
            Timed out waiting for the wallet to close the BLE session after sending the encrypted \
            DeviceResponse. The host Mac BLE helper validates the DeviceResponse out of process and \
            will fail the overall run separately if validation fails. Run \
            scripts/close_proximity/disclosure_mac_reader.swift \
            --qr-code <logged-qr-code> --expect-device-response-hex \(Self.macBleReaderExpectedDeviceResponseHex) \
            with the QR code logged by this test.
            """
        )

        let updatesAfterClose = await channel.receivedUpdates()
        guard
            let connectedIndex = updatesAfterClose.firstIndex(of: .connected),
            let sessionEstablishedIndex = updatesAfterClose.firstIndex(where: { update in
                if case .sessionEstablished = update {
                    return true
                }
                return false
            }),
            let closedIndex = updatesAfterClose.firstIndex(of: .closed)
        else {
            XCTFail(
                "Expected connected, SessionEstablished, and closed updates, got \(updatesAfterClose)"
            )
            return
        }

        XCTAssertLessThan(connectedIndex, sessionEstablishedIndex)
        XCTAssertLessThan(sessionEstablishedIndex, closedIndex)
        let isBleServerActiveAfterClose = await closeProximityDisclosure.isBleServerActiveForTesting()
        XCTAssertFalse(isBleServerActiveAfterClose)
    }

    func testStartQrHandoverFromTwoTasksReplacesPreviousSession() async throws {
        let closeProximityDisclosure = CloseProximityDisclosure()
        let firstChannel = TestChannel()
        let secondChannel = TestChannel()
        let startGate = StartGate(parties: 2)

        async let firstQrCode = startQrHandoverAfterGate(
            closeProximityDisclosure: closeProximityDisclosure,
            channel: firstChannel,
            startGate: startGate
        )
        async let secondQrCode = startQrHandoverAfterGate(
            closeProximityDisclosure: closeProximityDisclosure,
            channel: secondChannel,
            startGate: startGate
        )

        let resolvedFirstQrCode = try await firstQrCode
        let resolvedSecondQrCode = try await secondQrCode

        XCTAssertFalse(resolvedFirstQrCode.isEmpty)
        XCTAssertFalse(resolvedSecondQrCode.isEmpty)
        XCTAssertFalse(resolvedFirstQrCode.hasPrefix("mdoc:"))
        XCTAssertFalse(resolvedSecondQrCode.hasPrefix("mdoc:"))
        let isBleServerActiveAfterStart = await closeProximityDisclosure.isBleServerActiveForTesting()
        XCTAssertTrue(isBleServerActiveAfterStart)

        let firstClosed = await waitUntil(timeoutNanoseconds: 1_000_000_000) {
            await firstChannel.hasReceivedClosedUpdate()
        }
        let secondClosed = await waitUntil(timeoutNanoseconds: 1_000_000_000) {
            await secondChannel.hasReceivedClosedUpdate()
        }

        XCTAssertNotEqual(firstClosed, secondClosed)

        let replacedChannel = firstClosed ? firstChannel : secondChannel
        let activeChannel = firstClosed ? secondChannel : firstChannel

        let replacedUpdatesBeforeStop = await replacedChannel.receivedUpdates()
        let activeUpdatesBeforeStop = await activeChannel.receivedUpdates()
        XCTAssertEqual(replacedUpdatesBeforeStop, [.closed])
        XCTAssertTrue(activeUpdatesBeforeStop.isEmpty)

        try await closeProximityDisclosure.stopBleServer()

        let activeDidClose = await waitUntil { await activeChannel.hasReceivedClosedUpdate() }
        let activeUpdatesAfterStop = await activeChannel.receivedUpdates()
        let replacedUpdatesAfterStop = await replacedChannel.receivedUpdates()
        XCTAssertTrue(activeDidClose)
        XCTAssertEqual(activeUpdatesAfterStop, [.closed])
        XCTAssertEqual(replacedUpdatesAfterStop, [.closed])
        let isBleServerActiveAfterStop = await closeProximityDisclosure.isBleServerActiveForTesting()
        XCTAssertFalse(isBleServerActiveAfterStop)
    }

    private func waitUntil(
        timeoutNanoseconds: UInt64 = 5_000_000_000,
        pollIntervalNanoseconds: UInt64 = 50_000_000,
        condition: @escaping @Sendable () async -> Bool
    ) async -> Bool {
        let start = DispatchTime.now().uptimeNanoseconds

        while DispatchTime.now().uptimeNanoseconds - start < timeoutNanoseconds {
            if await condition() {
                return true
            }

            try? await Task.sleep(nanoseconds: pollIntervalNanoseconds)
        }

        return await condition()
    }

    private func startQrHandoverAfterGate(
        closeProximityDisclosure: CloseProximityDisclosure,
        channel: TestChannel,
        startGate: StartGate
    ) async throws -> String {
        await startGate.wait()
        return try await closeProximityDisclosure.startQrHandover(channel: channel)
    }

    private func advertiseAfterGate(
        transport: CloseProximityBleTransport,
        startGate: StartGate
    ) async -> AdvertiseAttemptResult {
        await startGate.wait()

        do {
            try await transport.advertise()
            return .started
        } catch {
            return .failed(error.localizedDescription)
        }
    }

    @MainActor
    private func runRawPeripheralManagerCleanupPermutationTest(
        name: String,
        actions: [RawCleanupAction]
    ) async throws {
        #if targetEnvironment(simulator)
            throw XCTSkip("CBPeripheralManager peripheral mode is not supported on the iOS Simulator")
        #endif

        let characterizer = RawPeripheralManagerCharacterizer()
        try await waitForRawPeripheralManagerPoweredOn(characterizer)

        let initialEvents = await startRawPeripheralManagerAdvertising(
            characterizer: characterizer,
            label: "\(name) initial start"
        )

        XCTAssertTrue(
            initialEvents.contains(where: { $0.contains("didStartAdvertising error=nil") }),
            "Expected initial advertising to start cleanly for permutation \(name), got \(initialEvents)"
        )

        NSLog(
            "Raw %@ cleanup begin isAdvertising=%@",
            name,
            String(characterizer.peripheralManager.isAdvertising)
        )

        for action in actions {
            switch action {
            case .stopAdvertising:
                characterizer.peripheralManager.stopAdvertising()
            case .removeAllServices:
                characterizer.peripheralManager.removeAllServices()
            }

            NSLog(
                "Raw %@ immediate isAdvertising after %@: %@",
                name,
                action.rawValue,
                String(characterizer.peripheralManager.isAdvertising)
            )
        }

        try await Task.sleep(nanoseconds: 200_000_000)
        NSLog(
            "Raw %@ isAdvertising after 200ms: %@",
            name,
            String(characterizer.peripheralManager.isAdvertising)
        )

        let restartEvents = await startRawPeripheralManagerAdvertising(
            characterizer: characterizer,
            label: "\(name) restart"
        )

        XCTAssertTrue(
            restartEvents.contains(where: { $0.contains("didStartAdvertising") }),
            "Expected a didStartAdvertising callback after cleanup permutation \(name), got \(restartEvents)"
        )

        characterizer.stop()
    }

    @MainActor
    private func waitForRawPeripheralManagerPoweredOn(
        _ characterizer: RawPeripheralManagerCharacterizer
    ) async throws {
        let didPowerOn = await waitUntil(timeoutNanoseconds: 5_000_000_000) {
            await characterizer.isPoweredOn()
        }

        guard didPowerOn else {
            let startupEvents = await characterizer.events()
            XCTFail("Expected CBPeripheralManager to become powered on, got \(startupEvents)")
            throw NSError(
                domain: "CloseProximityDisclosureTests",
                code: 10,
                userInfo: [NSLocalizedDescriptionKey: "CBPeripheralManager did not power on"]
            )
        }
    }

    @MainActor
    private func startRawPeripheralManagerAdvertising(
        characterizer: RawPeripheralManagerCharacterizer,
        label: String
    ) async -> [String] {
        let serviceUuid = CBUUID(string: UUID().uuidString)
        let service = characterizer.makeService(serviceUuid: serviceUuid)
        let baselineEventCount = await characterizer.eventCount()

        NSLog(
            "Raw CBPeripheralManager %@ begin service=%@",
            label,
            serviceUuid.uuidString
        )

        characterizer.peripheralManager.add(service)
        characterizer.peripheralManager.startAdvertising([
            CBAdvertisementDataServiceUUIDsKey: [serviceUuid],
        ])

        NSLog(
            "Raw CBPeripheralManager %@ immediate isAdvertising after start call: %@",
            label,
            String(characterizer.peripheralManager.isAdvertising)
        )

        _ = await waitUntil(timeoutNanoseconds: 2_000_000_000) {
            await characterizer.eventCount() >= baselineEventCount + 2
        }

        let events = Array((await characterizer.events()).dropFirst(baselineEventCount))
        for event in events {
            NSLog("Raw CBPeripheralManager %@ event: %@", label, event)
        }

        return events
    }

    private static func macBleReaderMarkerPayload(
        qrCode: String,
        expectedDeviceResponseHex: String? = nil
    ) -> String {
        if let expectedDeviceResponseHex {
            return
                "\(macBleReaderMarker){\"qr\":\"\(qrCode)\",\"expected_device_response_hex\":\"\(expectedDeviceResponseHex)\"}"
        }

        return "\(macBleReaderMarker){\"qr\":\"\(qrCode)\"}"
    }

    private func makeSessionEncryptionForRawDecryptCharacterization() -> SessionEncryption {
        let eDeviceKey = Crypto.shared.createEcPrivateKey(curve: .p256)
        let eReaderKey = Crypto.shared.createEcPrivateKey(curve: .p256)
        let encodedReaderCoseKey = Cbor.shared.encode(
            item: eReaderKey.publicKey.toCoseKey(additionalLabels: [:]).toDataItem()
        )
        // Keep the transcript setup minimal for this characterization test. The decrypt path only
        // needs stable shared bytes here; the malformed SessionData payload is the behavior under
        // test.
        let encodedSessionTranscript = cborEncodeArray([
            cborEncodeTagged(24, item: Data([0xF6])),
            cborEncodeTagged(24, item: Data(encodedReaderCoseKey.uint8Array())),
            Data([0xF6]),
        ])
        .map { $0 }
        .kotlinByteArray()

        return SessionEncryption(
            role: .mdoc,
            eSelfKey: eDeviceKey,
            remotePublicKey: eReaderKey.publicKey,
            encodedSessionTranscript: encodedSessionTranscript
        )
    }

    private func expectedSessionTranscript(forQrCode qrCode: String) throws -> [UInt8] {
        let encodedDeviceEngagement = try base64UrlDecodedData(qrCode)
        let readerPrivateKey = try P256.KeyAgreement.PrivateKey(
            rawRepresentation: try hexDecodedData(Self.macBleReaderDeterministicReaderPrivateKeyHex)
        )
        let readerPublicKeyBytes = readerPrivateKey.publicKey.x963Representation
        let encodedReaderCoseKey = cborEncodeMap([
            (cborEncodeUnsigned(1), cborEncodeUnsigned(2)),
            (cborEncodeNegative(-1), cborEncodeUnsigned(1)),
            (
                cborEncodeNegative(-2),
                cborEncodeByteString(Data(readerPublicKeyBytes.dropFirst().prefix(32)))
            ),
            (
                cborEncodeNegative(-3),
                cborEncodeByteString(Data(readerPublicKeyBytes.suffix(32)))
            ),
        ])

        let encodedSessionTranscript = cborEncodeArray([
            cborEncodeTagged(24, item: encodedDeviceEngagement),
            cborEncodeTagged(24, item: encodedReaderCoseKey),
            Data([0xF6]),
        ])
        return Array(encodedSessionTranscript)
    }

    private func hexDecodedData(_ hex: String) throws -> Data {
        guard hex.count.isMultiple(of: 2) else {
            throw NSError(
                domain: "CloseProximityDisclosureTests",
                code: 1,
                userInfo: [NSLocalizedDescriptionKey: "Invalid hex string"]
            )
        }

        var bytes: [UInt8] = []
        bytes.reserveCapacity(hex.count / 2)

        var index = hex.startIndex
        while index < hex.endIndex {
            let nextIndex = hex.index(index, offsetBy: 2)
            let byteString = hex[index..<nextIndex]
            guard let byte = UInt8(byteString, radix: 16) else {
                throw NSError(
                    domain: "CloseProximityDisclosureTests",
                    code: 2,
                    userInfo: [NSLocalizedDescriptionKey: "Invalid hex string"]
                )
            }
            bytes.append(byte)
            index = nextIndex
        }

        return Data(bytes)
    }

    private func base64UrlDecodedData(_ value: String) throws -> Data {
        let paddingLength = (4 - (value.count % 4)) % 4
        let padded =
            value
            .replacingOccurrences(of: "-", with: "+")
            .replacingOccurrences(of: "_", with: "/") + String(repeating: "=", count: paddingLength)

        guard let data = Data(base64Encoded: padded) else {
            throw NSError(
                domain: "CloseProximityDisclosureTests",
                code: 3,
                userInfo: [NSLocalizedDescriptionKey: "Invalid base64url payload"]
            )
        }
        return data
    }

    private func cborEncodeUnsigned(_ value: UInt64) -> Data {
        cborEncodeMajorType(0, value: value)
    }

    private func cborEncodeNegative(_ value: Int64) -> Data {
        precondition(value < 0)
        return cborEncodeMajorType(1, value: UInt64(-1 - value))
    }

    private func cborEncodeByteString(_ data: Data) -> Data {
        cborEncodeMajorType(2, value: UInt64(data.count)) + data
    }

    private func cborEncodeArray(_ items: [Data]) -> Data {
        var encoded = cborEncodeMajorType(4, value: UInt64(items.count))
        items.forEach { encoded += $0 }
        return encoded
    }

    private func cborEncodeMap(_ entries: [(Data, Data)]) -> Data {
        var encoded = cborEncodeMajorType(5, value: UInt64(entries.count))
        entries.forEach { key, value in
            encoded += key
            encoded += value
        }
        return encoded
    }

    private func cborEncodeTagged(_ tag: UInt64, item: Data) -> Data {
        cborEncodeMajorType(6, value: tag) + cborEncodeByteString(item)
    }

    private func cborEncodeMajorType(_ majorType: UInt8, value: UInt64) -> Data {
        precondition(majorType < 8)

        let prefix = majorType << 5
        switch value {
        case 0...23:
            return Data([prefix | UInt8(value)])
        case 24...0xFF:
            return Data([prefix | 24, UInt8(value)])
        case 0x100...0xFFFF:
            return Data([
                prefix | 25,
                UInt8((value >> 8) & 0xFF),
                UInt8(value & 0xFF),
            ])
        case 0x1_0000...0xFFFF_FFFF:
            return Data([
                prefix | 26,
                UInt8((value >> 24) & 0xFF),
                UInt8((value >> 16) & 0xFF),
                UInt8((value >> 8) & 0xFF),
                UInt8(value & 0xFF),
            ])
        default:
            return Data([
                prefix | 27,
                UInt8((value >> 56) & 0xFF),
                UInt8((value >> 48) & 0xFF),
                UInt8((value >> 40) & 0xFF),
                UInt8((value >> 32) & 0xFF),
                UInt8((value >> 24) & 0xFF),
                UInt8((value >> 16) & 0xFF),
                UInt8((value >> 8) & 0xFF),
                UInt8(value & 0xFF),
            ])
        }
    }
}

private extension CBManagerState {
    var summary: String {
        switch self {
        case .unknown:
            return "unknown"
        case .resetting:
            return "resetting"
        case .unsupported:
            return "unsupported"
        case .unauthorized:
            return "unauthorized"
        case .poweredOff:
            return "poweredOff"
        case .poweredOn:
            return "poweredOn"
        @unknown default:
            return "unknownDefault(\(rawValue))"
        }
    }
}
