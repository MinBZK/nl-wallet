//
//  CloseProximityDisclosureTests.swift
//  Integration Tests
//
//  Created by The Wallet Developers on 06/03/2026.
//

import CryptoKit
import Foundation
@preconcurrency import Multipaz
import XCTest

@testable import PlatformSupport

final class CloseProximityDisclosureTests: XCTestCase {
    private enum RecordedUpdate: Equatable, Sendable {
        case connecting
        case sessionEstablished(sessionTranscript: [UInt8], deviceRequest: [UInt8])
        case closed
        case other
    }

    private final class TestChannel: CloseProximityDisclosureChannel, @unchecked Sendable {
        private actor State {
            private var updates: [RecordedUpdate] = []

            func record(update: CloseProximityDisclosureUpdate) {
                switch update {
                case .connecting:
                    updates.append(.connecting)
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

            func hasReceivedConnectingUpdate() -> Bool {
                updates.contains(.connecting)
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

        func hasReceivedConnectingUpdate() async -> Bool {
            await state.hasReceivedConnectingUpdate()
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
            let connectingIndex = updatesAfterClose.firstIndex(of: .connecting),
            let sessionEstablishedIndex = updatesAfterClose.firstIndex(where: { update in
                if case .sessionEstablished = update {
                    return true
                }
                return false
            }),
            let closedIndex = updatesAfterClose.firstIndex(of: .closed)
        else {
            XCTFail(
                "Expected connecting, SessionEstablished, and closed updates, got \(updatesAfterClose)"
            )
            return
        }

        XCTAssertLessThan(connectingIndex, sessionEstablishedIndex)
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
