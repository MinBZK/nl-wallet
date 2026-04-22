#!/usr/bin/env swift

import CoreBluetooth
import CryptoKit
import Foundation

private struct Configuration {
    let serviceUuid: CBUUID
    let timeout: TimeInterval
    let sessionMessage: Data?
    let deviceResponseHandling: DeviceResponseHandling?

    static let defaultServiceUuidString = "08c5f8e7-3078-4cc3-b6f4-1f861a7f67e9"
    static let defaultTimeout: TimeInterval = 30
    static let deterministicReaderPrivateKeyHex =
        "de3b4b9e5f72dd9b58406ae3091434da48a6f9fd010d88fcb0958e2cebec947c"
    static let sessionEstablishedDeviceRequest = Data([0x01, 0x02, 0x03])
    static let sessionTerminationStatusCode: Int64 = 20
    static let bleEndByte: UInt8 = 0x02
    static let deviceResponseHexMarker = "CLOSE_PROXIMITY_DEVICE_RESPONSE_HEX="
}

private struct EngagementData {
    let encodedDeviceEngagement: Data
    let serviceUuid: CBUUID
    let eDeviceKey: P256.KeyAgreement.PublicKey
}

private struct SessionEstablishedData {
    let message: Data
    let encodedSessionTranscript: Data
}

private struct DeviceResponseHandling {
    let expectedPlaintext: Data?
    let shouldPrintPlaintextHex: Bool
    let encodedSessionTranscript: Data
    let eDeviceKey: P256.KeyAgreement.PublicKey
}

private enum ArgumentError: LocalizedError {
    case missingValue(String)
    case invalidTimeout(String)
    case invalidQrCode(String)
    case invalidHex(String)
    case invalidConfiguration(String)
    case invalidDeterministicKey
    case unsupportedCurve(String)
    case externalToolFailure(String)
    case unexpectedArgument(String)

    var errorDescription: String? {
        switch self {
        case .missingValue(let flag):
            return "Missing value for \(flag)"
        case .invalidTimeout(let value):
            return "Invalid timeout value: \(value)"
        case .invalidQrCode(let reason):
            return "Invalid QR code/device engagement: \(reason)"
        case .invalidHex(let value):
            return "Invalid hex value: \(value)"
        case .invalidConfiguration(let reason):
            return "Invalid configuration: \(reason)"
        case .invalidDeterministicKey:
            return "Invalid deterministic reader private key"
        case .unsupportedCurve(let reason):
            return "Unsupported curve: \(reason)"
        case .externalToolFailure(let reason):
            return "External tool failed: \(reason)"
        case .unexpectedArgument(let argument):
            return "Unexpected argument: \(argument)"
        }
    }
}

private enum ValidationError: LocalizedError {
    case invalidDeviceResponse(String)

    var errorDescription: String? {
        switch self {
        case .invalidDeviceResponse(let reason):
            return reason
        }
    }
}

private indirect enum Cbor {
    case int(Int64)
    case bytes(Data)
    case text(String)
    case array([Cbor])
    case map([(Cbor, Cbor)])
    case tagged(UInt64, Cbor)
    case bool(Bool)
    case null
}

extension Cbor {
    fileprivate var int: Int64? {
        if case .int(let value) = self { return value }
        return nil
    }

    fileprivate var bytes: Data? {
        if case .bytes(let value) = self { return value }
        return nil
    }

    fileprivate var text: String? {
        if case .text(let value) = self { return value }
        return nil
    }

    fileprivate var array: [Cbor]? {
        if case .array(let value) = self { return value }
        return nil
    }

    fileprivate var map: [(Cbor, Cbor)]? {
        if case .map(let value) = self { return value }
        return nil
    }
}

private struct CborParser {
    private let bytes: [UInt8]
    private var index = 0

    init(data: Data) { bytes = Array(data) }

    mutating func decode() throws -> Cbor {
        guard index < bytes.count else {
            throw ArgumentError.invalidQrCode("unexpected end of CBOR input")
        }

        let initialByte = bytes[index]
        index += 1

        let majorType = initialByte >> 5
        let additionalInfo = initialByte & 0x1F

        switch majorType {
        case 0:
            return .int(Int64(try readLength(additionalInfo)))
        case 1:
            return .int(-1 - Int64(try readLength(additionalInfo)))
        case 2:
            return .bytes(try readData(count: Int(try readLength(additionalInfo))))
        case 3:
            let data = try readData(count: Int(try readLength(additionalInfo)))
            guard let string = String(data: data, encoding: .utf8) else {
                throw ArgumentError.invalidQrCode("invalid UTF-8 text string")
            }
            return .text(string)
        case 4:
            let count = Int(try readLength(additionalInfo))
            return .array(try (0..<count).map { _ in try decode() })
        case 5:
            let count = Int(try readLength(additionalInfo))
            return .map(try (0..<count).map { _ in (try decode(), try decode()) })
        case 6:
            return .tagged(try readLength(additionalInfo), try decode())
        case 7:
            switch additionalInfo {
            case 20: return .bool(false)
            case 21: return .bool(true)
            case 22: return .null
            default:
                throw ArgumentError.invalidQrCode("unsupported simple value \(additionalInfo)")
            }
        default:
            throw ArgumentError.invalidQrCode("unsupported CBOR major type \(majorType)")
        }
    }

    private mutating func readLength(_ additionalInfo: UInt8) throws -> UInt64 {
        switch additionalInfo {
        case 0...23:
            return UInt64(additionalInfo)
        case 24:
            return UInt64(try readByte())
        case 25:
            return try readFixedWidthInteger(byteCount: 2)
        case 26:
            return try readFixedWidthInteger(byteCount: 4)
        case 27:
            return try readFixedWidthInteger(byteCount: 8)
        default:
            throw ArgumentError.invalidQrCode("unsupported indefinite-length CBOR")
        }
    }

    private mutating func readFixedWidthInteger(byteCount: Int) throws -> UInt64 {
        try readData(count: byteCount).reduce(0) { ($0 << 8) | UInt64($1) }
    }

    private mutating func readByte() throws -> UInt8 {
        guard index < bytes.count else {
            throw ArgumentError.invalidQrCode("unexpected end of CBOR input")
        }
        let value = bytes[index]
        index += 1
        return value
    }

    private mutating func readData(count: Int) throws -> Data {
        guard index + count <= bytes.count else {
            throw ArgumentError.invalidQrCode("unexpected end of CBOR input")
        }

        let data = Data(bytes[index..<(index + count)])
        index += count
        return data
    }
}

private func cborEncode(_ value: Cbor) -> Data {
    switch value {
    case .int(let value) where value >= 0:
        return cborHead(majorType: 0, value: UInt64(value))
    case .int(let value):
        return cborHead(majorType: 1, value: UInt64(-1 - value))
    case .bytes(let data):
        return cborHead(majorType: 2, value: UInt64(data.count)) + data
    case .text(let string):
        let data = Data(string.utf8)
        return cborHead(majorType: 3, value: UInt64(data.count)) + data
    case .array(let values):
        return values.reduce(cborHead(majorType: 4, value: UInt64(values.count))) {
            $0 + cborEncode($1)
        }
    case .map(let entries):
        return entries.reduce(cborHead(majorType: 5, value: UInt64(entries.count))) {
            $0 + cborEncode($1.0) + cborEncode($1.1)
        }
    case .tagged(let tag, let value):
        return cborHead(majorType: 6, value: tag) + cborEncode(value)
    case .bool(false):
        return Data([0xF4])
    case .bool(true):
        return Data([0xF5])
    case .null:
        return Data([0xF6])
    }
}

private func cborHead(majorType: UInt8, value: UInt64) -> Data {
    precondition(majorType < 8)

    let prefix = majorType << 5
    switch value {
    case 0...23:
        return Data([prefix | UInt8(value)])
    case 24...0xFF:
        return Data([prefix | 24, UInt8(value)])
    case 0x100...0xFFFF:
        return Data([prefix | 25, UInt8((value >> 8) & 0xFF), UInt8(value & 0xFF)])
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

private func parseConfiguration(arguments: [String]) throws -> Configuration {
    var serviceUuidString = Configuration.defaultServiceUuidString
    var timeout = Configuration.defaultTimeout
    var qrCode: String?
    var deviceRequestHex: String?
    var expectedDeviceResponseHex: String?
    var readerCaCrtFile: String?
    var readerCaKeyFile: String?
    var readerAuthFile: String?
    var printDeviceResponseHex = false
    var index = 0

    while index < arguments.count {
        switch arguments[index] {
        case "--service-uuid":
            index += 1
            guard index < arguments.count else {
                throw ArgumentError.missingValue("--service-uuid")
            }
            serviceUuidString = arguments[index]
        case "--timeout":
            index += 1
            guard index < arguments.count else { throw ArgumentError.missingValue("--timeout") }
            guard let parsedTimeout = TimeInterval(arguments[index]), parsedTimeout > 0 else {
                throw ArgumentError.invalidTimeout(arguments[index])
            }
            timeout = parsedTimeout
        case "--qr-code":
            index += 1
            guard index < arguments.count else { throw ArgumentError.missingValue("--qr-code") }
            qrCode = arguments[index]
        case "--device-request-hex":
            index += 1
            guard index < arguments.count else {
                throw ArgumentError.missingValue("--device-request-hex")
            }
            deviceRequestHex = arguments[index]
        case "--expect-device-response-hex":
            index += 1
            guard index < arguments.count else {
                throw ArgumentError.missingValue("--expect-device-response-hex")
            }
            expectedDeviceResponseHex = arguments[index]
        case "--reader-ca-crt-file":
            index += 1
            guard index < arguments.count else {
                throw ArgumentError.missingValue("--reader-ca-crt-file")
            }
            readerCaCrtFile = arguments[index]
        case "--reader-ca-key-file":
            index += 1
            guard index < arguments.count else {
                throw ArgumentError.missingValue("--reader-ca-key-file")
            }
            readerCaKeyFile = arguments[index]
        case "--reader-auth-file":
            index += 1
            guard index < arguments.count else {
                throw ArgumentError.missingValue("--reader-auth-file")
            }
            readerAuthFile = arguments[index]
        case "--print-device-response-hex":
            printDeviceResponseHex = true
        case "--help":
            print(
                """
                Usage: swift scripts/close_proximity/disclosure_mac_reader.swift [options]

                  --service-uuid <uuid>  BLE service UUID to scan for in connect-only mode.
                                         Default: \(Configuration.defaultServiceUuidString)
                  --qr-code <base64url>  Device-engagement QR payload. When set, the helper
                                         writes a reader session-establishment message.
                  --device-request-hex <hex>
                                         Override the plaintext DeviceRequest that will be
                                         encrypted into the first reader message.
                  --reader-ca-crt-file <path>
                  --reader-ca-key-file <path>
                  --reader-auth-file <path>
                                         Generate a real signed DeviceRequest via
                                         `wallet_ca reader-device-request` using the provided
                                         reader CA material and reader_auth.json. This needs to be in pem format.
                  --expect-device-response-hex <hex>
                                         After SessionEstablished, wait for the holder to send
                                         an encrypted DeviceResponse with status 20, validate the
                                         decrypted plaintext, then send BLE end.
                  --print-device-response-hex
                                         Print the decrypted DeviceResponse plaintext as
                                         \(Configuration.deviceResponseHexMarker)<hex>.
                  --timeout <seconds>    How long to wait before giving up.
                                         Default: \(Int(Configuration.defaultTimeout))
                """
            )
            exit(0)
        default:
            throw ArgumentError.unexpectedArgument(arguments[index])
        }

        index += 1
    }

    if (expectedDeviceResponseHex != nil || deviceRequestHex != nil || printDeviceResponseHex)
        && qrCode == nil
    {
        throw ArgumentError.invalidConfiguration(
            "--device-request-hex, --expect-device-response-hex, and --print-device-response-hex require --qr-code"
        )
    }

    let usesReaderMaterial =
        readerCaCrtFile != nil || readerCaKeyFile != nil || readerAuthFile != nil
    if usesReaderMaterial && qrCode == nil {
        throw ArgumentError.invalidConfiguration(
            "--reader-ca-crt-file, --reader-ca-key-file, and --reader-auth-file require --qr-code"
        )
    }
    if usesReaderMaterial
        && (readerCaCrtFile == nil || readerCaKeyFile == nil || readerAuthFile == nil)
    {
        throw ArgumentError.invalidConfiguration(
            "--reader-ca-crt-file, --reader-ca-key-file, and --reader-auth-file must be provided together"
        )
    }
    if usesReaderMaterial && deviceRequestHex != nil {
        throw ArgumentError.invalidConfiguration(
            "--device-request-hex cannot be combined with --reader-ca-crt-file/--reader-ca-key-file/--reader-auth-file"
        )
    }

    if let qrCode {
        let engagement = try parseDeviceEngagement(fromBase64Url: qrCode)
        let encodedSessionTranscript = try buildEncodedSessionTranscript(engagement: engagement)
        let deviceRequest: Data
        if let deviceRequestHex {
            deviceRequest = try hexDecodedData(deviceRequestHex)
        } else if let readerCaCrtFile, let readerCaKeyFile, let readerAuthFile {
            deviceRequest = try generateReaderDeviceRequest(
                encodedSessionTranscript: encodedSessionTranscript,
                readerCaCrtFile: readerCaCrtFile,
                readerCaKeyFile: readerCaKeyFile,
                readerAuthFile: readerAuthFile
            )
        } else {
            deviceRequest = Configuration.sessionEstablishedDeviceRequest
        }
        let sessionEstablishedData = try buildSessionEstablishedMessage(
            engagement: engagement,
            deviceRequest: deviceRequest
        )

        let deviceResponseHandling: DeviceResponseHandling?
        if expectedDeviceResponseHex != nil || printDeviceResponseHex {
            let expectedPlaintext: Data?
            if let expectedDeviceResponseHex {
                expectedPlaintext = try hexDecodedData(expectedDeviceResponseHex)
            } else {
                expectedPlaintext = nil
            }
            deviceResponseHandling = DeviceResponseHandling(
                expectedPlaintext: expectedPlaintext,
                shouldPrintPlaintextHex: printDeviceResponseHex,
                encodedSessionTranscript: sessionEstablishedData.encodedSessionTranscript,
                eDeviceKey: engagement.eDeviceKey
            )
        } else {
            deviceResponseHandling = nil
        }
        return Configuration(
            serviceUuid: engagement.serviceUuid,
            timeout: timeout,
            sessionMessage: sessionEstablishedData.message,
            deviceResponseHandling: deviceResponseHandling
        )
    }

    return Configuration(
        serviceUuid: CBUUID(string: serviceUuidString),
        timeout: timeout,
        sessionMessage: nil,
        deviceResponseHandling: nil
    )
}

private func parseDeviceEngagement(fromBase64Url qrCode: String) throws -> EngagementData {
    let encodedDeviceEngagement = try base64UrlDecodedData(qrCode)
    var parser = CborParser(data: encodedDeviceEngagement)
    let root = try parser.decode()

    guard let rootEntries = root.map else {
        throw ArgumentError.invalidQrCode("device engagement must be a CBOR map")
    }
    guard
        let security = lookup(rootEntries, intKey: 1)?.array,
        security.count >= 2,
        case .tagged(24, .bytes(let encodedEDeviceKey)) = security[1]
    else {
        throw ArgumentError.invalidQrCode("device engagement security block is malformed")
    }

    let eDeviceKey = try parseP256PublicKey(fromCoseKey: encodedEDeviceKey)

    guard
        let connectionMethods = lookup(rootEntries, intKey: 2)?.array,
        let firstConnectionMethod = connectionMethods.first?.array,
        firstConnectionMethod.count >= 3,
        let options = firstConnectionMethod[2].map
    else {
        throw ArgumentError.invalidQrCode("device engagement connection method is malformed")
    }

    guard
        let serviceUuidBytes = lookup(options, intKey: 10)?.bytes
            ?? lookup(options, intKey: 11)?.bytes
    else {
        throw ArgumentError.invalidQrCode("BLE service UUID not found in device engagement")
    }

    return EngagementData(
        encodedDeviceEngagement: encodedDeviceEngagement,
        serviceUuid: CBUUID(string: try uuid(from: serviceUuidBytes).uuidString),
        eDeviceKey: eDeviceKey
    )
}

private func buildEncodedSessionTranscript(engagement: EngagementData) throws -> Data {
    let readerPrivateKey = try deterministicReaderPrivateKey()
    let encodedReaderCoseKey = encodeP256PublicKeyAsCoseKey(readerPrivateKey.publicKey)
    return cborEncode(
        .array([
            .tagged(24, .bytes(engagement.encodedDeviceEngagement)),
            .tagged(24, .bytes(encodedReaderCoseKey)),
            .null,
        ])
    )
}

private func buildSessionEstablishedMessage(
    engagement: EngagementData,
    deviceRequest: Data
) throws -> SessionEstablishedData {
    let readerPrivateKey = try deterministicReaderPrivateKey()
    let encodedReaderCoseKey = encodeP256PublicKeyAsCoseKey(readerPrivateKey.publicKey)
    let encodedSessionTranscript = try buildEncodedSessionTranscript(engagement: engagement)

    let skReader = try deriveSessionKey(
        eDeviceKey: engagement.eDeviceKey,
        encodedSessionTranscript: encodedSessionTranscript,
        sharedInfo: "SKReader"
    )
    let nonce = try AES.GCM.Nonce(
        data: Data([
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ]))
    let sealedBox = try AES.GCM.seal(
        deviceRequest,
        using: skReader,
        nonce: nonce
    )
    let ciphertext = sealedBox.ciphertext + sealedBox.tag

    return SessionEstablishedData(
        message: cborEncode(
            .map([
                (.text("eReaderKey"), .tagged(24, .bytes(encodedReaderCoseKey))),
                (.text("data"), .bytes(ciphertext)),
            ])
        ),
        encodedSessionTranscript: encodedSessionTranscript
    )
}

private func deriveSessionKey(
    eDeviceKey: P256.KeyAgreement.PublicKey,
    encodedSessionTranscript: Data,
    sharedInfo: String
) throws -> SymmetricKey {
    let readerPrivateKey = try deterministicReaderPrivateKey()
    let salt = Data(SHA256.hash(data: cborEncode(.tagged(24, .bytes(encodedSessionTranscript)))))
    let sharedSecret = try readerPrivateKey.sharedSecretFromKeyAgreement(with: eDeviceKey)
    return sharedSecret.hkdfDerivedSymmetricKey(
        using: SHA256.self,
        salt: salt,
        sharedInfo: Data(sharedInfo.utf8),
        outputByteCount: 32
    )
}

private func deterministicReaderPrivateKey() throws -> P256.KeyAgreement.PrivateKey {
    let hex = Configuration.deterministicReaderPrivateKeyHex
    guard hex.count.isMultiple(of: 2) else {
        throw ArgumentError.invalidDeterministicKey
    }

    var bytes: [UInt8] = []
    bytes.reserveCapacity(hex.count / 2)

    var index = hex.startIndex
    while index < hex.endIndex {
        let nextIndex = hex.index(index, offsetBy: 2)
        guard let byte = UInt8(hex[index..<nextIndex], radix: 16) else {
            throw ArgumentError.invalidDeterministicKey
        }
        bytes.append(byte)
        index = nextIndex
    }

    do {
        return try P256.KeyAgreement.PrivateKey(rawRepresentation: Data(bytes))
    } catch {
        throw ArgumentError.invalidDeterministicKey
    }
}

private func encodeP256PublicKeyAsCoseKey(_ publicKey: P256.KeyAgreement.PublicKey) -> Data {
    let x963 = publicKey.x963Representation
    precondition(x963.count == 65 && x963.first == 0x04)

    return cborEncode(
        .map([
            (.int(1), .int(2)),
            (.int(-1), .int(1)),
            (.int(-2), .bytes(Data(x963.dropFirst().prefix(32)))),
            (.int(-3), .bytes(Data(x963.suffix(32)))),
        ])
    )
}

private func parseP256PublicKey(fromCoseKey encodedCoseKey: Data) throws
    -> P256.KeyAgreement.PublicKey
{
    var parser = CborParser(data: encodedCoseKey)
    let root = try parser.decode()

    guard let entries = root.map else {
        throw ArgumentError.invalidQrCode("COSE key must be a CBOR map")
    }
    guard lookup(entries, intKey: 1)?.int == 2 else {
        throw ArgumentError.invalidQrCode("unsupported COSE key type")
    }
    guard lookup(entries, intKey: -1)?.int == 1 else {
        throw ArgumentError.unsupportedCurve("only P-256 is supported by this helper")
    }
    guard
        let x = lookup(entries, intKey: -2)?.bytes,
        let y = lookup(entries, intKey: -3)?.bytes,
        x.count == 32,
        y.count == 32
    else {
        throw ArgumentError.invalidQrCode("COSE key coordinates are malformed")
    }

    return try P256.KeyAgreement.PublicKey(x963Representation: Data([0x04]) + x + y)
}

private func lookup(_ entries: [(Cbor, Cbor)], intKey: Int64) -> Cbor? {
    entries.first { $0.0.int == intKey }?.1
}

private func lookup(_ entries: [(Cbor, Cbor)], textKey: String) -> Cbor? {
    entries.first { $0.0.text == textKey }?.1
}

private func uuid(from data: Data) throws -> UUID {
    guard data.count == 16 else {
        throw ArgumentError.invalidQrCode("BLE UUID must be 16 bytes")
    }

    let bytes = Array(data)
    return UUID(
        uuid: (
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
            bytes[8], bytes[9], bytes[10], bytes[11],
            bytes[12], bytes[13], bytes[14], bytes[15]
        ))
}

private func base64UrlDecodedData(_ value: String) throws -> Data {
    let paddingLength = (4 - (value.count % 4)) % 4
    let padded =
        value
        .replacingOccurrences(of: "-", with: "+")
        .replacingOccurrences(of: "_", with: "/") + String(repeating: "=", count: paddingLength)

    guard let data = Data(base64Encoded: padded) else {
        throw ArgumentError.invalidQrCode("invalid base64url payload")
    }
    return data
}

private func hexDecodedData(_ hex: String) throws -> Data {
    guard hex.count.isMultiple(of: 2) else {
        throw ArgumentError.invalidHex(hex)
    }

    var bytes: [UInt8] = []
    bytes.reserveCapacity(hex.count / 2)

    var index = hex.startIndex
    while index < hex.endIndex {
        let nextIndex = hex.index(index, offsetBy: 2)
        guard let byte = UInt8(hex[index..<nextIndex], radix: 16) else {
            throw ArgumentError.invalidHex(hex)
        }
        bytes.append(byte)
        index = nextIndex
    }

    return Data(bytes)
}

private func hexEncodedData(_ data: Data) -> String {
    data.map { String(format: "%02x", $0) }.joined()
}

private func generateReaderDeviceRequest(
    encodedSessionTranscript: Data,
    readerCaCrtFile: String,
    readerCaKeyFile: String,
    readerAuthFile: String
) throws -> Data {
    let process = Process()
    process.executableURL = URL(fileURLWithPath: "/usr/bin/env")
    process.arguments = [
        "cargo",
        "run",
        "--quiet",
        "--manifest-path",
        "wallet_core/Cargo.toml",
        "--bin",
        "wallet_ca",
        "--",
        "reader-device-request",
        "--ca-crt-file",
        readerCaCrtFile,
        "--ca-key-file",
        readerCaKeyFile,
        "--reader-auth-file",
        readerAuthFile,
        "--session-transcript-hex",
        hexEncodedData(encodedSessionTranscript),
    ]
    process.currentDirectoryURL = URL(fileURLWithPath: FileManager.default.currentDirectoryPath)

    let stdout = Pipe()
    let stderr = Pipe()
    process.standardOutput = stdout
    process.standardError = stderr

    do {
        try process.run()
    } catch {
        throw ArgumentError.externalToolFailure(error.localizedDescription)
    }

    process.waitUntilExit()

    let stdoutData = stdout.fileHandleForReading.readDataToEndOfFile()
    let stderrData = stderr.fileHandleForReading.readDataToEndOfFile()
    let stdoutText = String(decoding: stdoutData, as: UTF8.self).trimmingCharacters(
        in: .whitespacesAndNewlines)
    let stderrText = String(decoding: stderrData, as: UTF8.self).trimmingCharacters(
        in: .whitespacesAndNewlines)

    guard process.terminationStatus == 0 else {
        throw ArgumentError.externalToolFailure(
            stderrText.isEmpty
                ? "wallet_ca exited with code \(process.terminationStatus)" : stderrText)
    }
    guard !stdoutText.isEmpty else {
        throw ArgumentError.externalToolFailure(
            "wallet_ca did not return a DeviceRequest hex payload")
    }

    return try hexDecodedData(stdoutText)
}

private func decryptedDeviceResponsePlaintext(
    _ message: Data,
    handling: DeviceResponseHandling
) throws -> Data {
    var parser = CborParser(data: message)
    let root = try parser.decode()

    guard let entries = root.map else {
        throw ValidationError.invalidDeviceResponse(
            "Reader received a device response that is not a CBOR map")
    }
    let status = lookup(entries, textKey: "status")?.int
    guard let encryptedPayload = lookup(entries, textKey: "data")?.bytes else {
        if let status {
            throw ValidationError.invalidDeviceResponse(
                "Reader received status \(status) instead of an encrypted DeviceResponse"
            )
        }
        throw ValidationError.invalidDeviceResponse(
            "Reader received a device response without a data field")
    }
    guard status == Configuration.sessionTerminationStatusCode else {
        throw ValidationError.invalidDeviceResponse(
            "reader received status \(status ?? -1) instead of \(Configuration.sessionTerminationStatusCode)"
        )
    }
    guard encryptedPayload.count >= 16 else {
        throw ValidationError.invalidDeviceResponse(
            "Reader received a device response with malformed ciphertext")
    }

    let skDevice = try deriveSessionKey(
        eDeviceKey: handling.eDeviceKey,
        encodedSessionTranscript: handling.encodedSessionTranscript,
        sharedInfo: "SKDevice"
    )
    let nonce = try AES.GCM.Nonce(
        data: Data([
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x01,
        ]))
    let ciphertext = encryptedPayload.dropLast(16)
    let tag = encryptedPayload.suffix(16)
    let sealedBox = try AES.GCM.SealedBox(
        nonce: nonce,
        ciphertext: Data(ciphertext),
        tag: Data(tag)
    )
    return try AES.GCM.open(sealedBox, using: skDevice)
}

private final class CloseProximityReader: NSObject, CBCentralManagerDelegate, CBPeripheralDelegate {
    private let serviceUuid: CBUUID
    private let timeout: TimeInterval
    private let sessionMessage: Data?
    private let deviceResponseHandling: DeviceResponseHandling?
    private let stateCharacteristicUuid = CBUUID(string: "00000001-a123-48ce-896b-4c76973373e6")
    private let clientToServerCharacteristicUuid = CBUUID(
        string: "00000002-a123-48ce-896b-4c76973373e6")
    private let serverToClientCharacteristicUuid = CBUUID(
        string: "00000003-a123-48ce-896b-4c76973373e6")

    private var centralManager: CBCentralManager?
    private var activePeripheral: CBPeripheral?
    private var stateCharacteristic: CBCharacteristic?
    private var hasTriggeredStart = false
    private var hasCompletedDeviceResponseFlow = false
    private var deviceResponseBuffer = Data()
    private var serverToClientChunkCount = 0
    private var serverToClientBufferedByteCount = 0
    private var timeoutTimer: Timer?

    init(configuration: Configuration) {
        serviceUuid = configuration.serviceUuid
        timeout = configuration.timeout
        sessionMessage = configuration.sessionMessage
        deviceResponseHandling = configuration.deviceResponseHandling
        super.init()
    }

    func run() -> Never {
        if sessionMessage == nil {
            log("Waiting for service \(serviceUuid.uuidString)")
        } else if let deviceResponseHandling {
            if deviceResponseHandling.expectedPlaintext == nil {
                log(
                    "Waiting for service \(serviceUuid.uuidString) to capture a decrypted DeviceResponse"
                )
            } else {
                log(
                    "Waiting for service \(serviceUuid.uuidString) to validate DeviceResponse test message"
                )
            }
        } else {
            log(
                "Waiting for service \(serviceUuid.uuidString) to send SessionEstablished test message"
            )
        }

        timeoutTimer = Timer.scheduledTimer(withTimeInterval: timeout, repeats: false) {
            [weak self] _ in
            guard let self else { return }
            self.fail(
                "Timed out after \(Int(self.timeout)) seconds "
                    + "(serverToClientChunkCount=\(self.serverToClientChunkCount), "
                    + "bufferedBytes=\(self.serverToClientBufferedByteCount))"
            )
        }

        centralManager = CBCentralManager(delegate: self, queue: nil)
        RunLoop.main.run()
        fatalError("RunLoop.main.run() should never return")
    }

    func centralManagerDidUpdateState(_ central: CBCentralManager) {
        switch central.state {
        case .poweredOn:
            log("Bluetooth powered on, scanning")
            scan()
        case .unauthorized:
            fail(
                "Bluetooth access is unauthorized. Grant Terminal Bluetooth access in System Settings."
            )
        case .unsupported:
            fail("Bluetooth LE is unsupported on this Mac.")
        case .poweredOff:
            fail("Bluetooth is powered off.")
        case .resetting:
            log("Bluetooth is resetting")
        case .unknown:
            log("Bluetooth state is unknown")
        @unknown default:
            fail("Unexpected Bluetooth state: \(central.state.rawValue)")
        }
    }

    func centralManager(
        _ central: CBCentralManager,
        didDiscover peripheral: CBPeripheral,
        advertisementData: [String: Any],
        rssi RSSI: NSNumber
    ) {
        guard activePeripheral == nil else { return }

        activePeripheral = peripheral
        peripheral.delegate = self

        log("Discovered \(peripheral.identifier.uuidString), connecting")
        central.stopScan()
        central.connect(peripheral, options: nil)
    }

    func centralManager(_ central: CBCentralManager, didConnect peripheral: CBPeripheral) {
        log("Connected to \(peripheral.identifier.uuidString), discovering services")
        peripheral.discoverServices([serviceUuid])
    }

    func centralManager(
        _ central: CBCentralManager, didFailToConnect peripheral: CBPeripheral, error: Error?
    ) {
        log("Failed to connect: \(error?.localizedDescription ?? "unknown error")")
        activePeripheral = nil
        restartScan()
    }

    func centralManager(
        _ central: CBCentralManager, didDisconnectPeripheral peripheral: CBPeripheral, error: Error?
    ) {
        if hasTriggeredStart {
            return
        }

        log("Disconnected before start byte write: \(error?.localizedDescription ?? "no error")")
        activePeripheral = nil
        restartScan()
    }

    func peripheral(_ peripheral: CBPeripheral, didDiscoverServices error: Error?) {
        if let error {
            log("Service discovery failed: \(error.localizedDescription)")
            centralManager?.cancelPeripheralConnection(peripheral)
            return
        }

        guard let service = peripheral.services?.first(where: { $0.uuid == serviceUuid }) else {
            log("Target service \(serviceUuid.uuidString) not found")
            centralManager?.cancelPeripheralConnection(peripheral)
            return
        }

        log("Discovered service, discovering characteristics")
        var characteristicUuids = [stateCharacteristicUuid]
        if sessionMessage != nil {
            characteristicUuids.append(clientToServerCharacteristicUuid)
        }
        if deviceResponseHandling != nil {
            characteristicUuids.append(serverToClientCharacteristicUuid)
        }
        peripheral.discoverCharacteristics(characteristicUuids, for: service)
    }

    func peripheral(
        _ peripheral: CBPeripheral,
        didDiscoverCharacteristicsFor service: CBService,
        error: Error?
    ) {
        if let error {
            log("Characteristic discovery failed: \(error.localizedDescription)")
            centralManager?.cancelPeripheralConnection(peripheral)
            return
        }

        guard
            let stateCharacteristic = service.characteristics?.first(where: {
                $0.uuid == stateCharacteristicUuid
            })
        else {
            log("State characteristic \(stateCharacteristicUuid.uuidString) not found")
            centralManager?.cancelPeripheralConnection(peripheral)
            return
        }
        self.stateCharacteristic = stateCharacteristic

        if deviceResponseHandling != nil {
            guard
                let serverToClientCharacteristic = service.characteristics?.first(
                    where: { $0.uuid == serverToClientCharacteristicUuid }
                )
            else {
                log(
                    "Server-to-client characteristic \(serverToClientCharacteristicUuid.uuidString) not found"
                )
                centralManager?.cancelPeripheralConnection(peripheral)
                return
            }
            peripheral.setNotifyValue(true, for: serverToClientCharacteristic)
            log("Subscribed to server-to-client characteristic for DeviceResponse validation")
        }

        log("Writing start byte to state characteristic")
        hasTriggeredStart = true
        peripheral.writeValue(Data([0x01]), for: stateCharacteristic, type: .withoutResponse)

        guard let sessionMessage else {
            DispatchQueue.main.asyncAfter(deadline: .now() + 1) { [weak self] in
                self?.succeed("Start byte written successfully")
            }
            return
        }

        guard
            let clientToServerCharacteristic = service.characteristics?.first(
                where: { $0.uuid == clientToServerCharacteristicUuid }
            )
        else {
            log(
                "Client-to-server characteristic \(clientToServerCharacteristicUuid.uuidString) not found"
            )
            centralManager?.cancelPeripheralConnection(peripheral)
            return
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) { [weak self, weak peripheral] in
            guard let self, let peripheral else { return }

            self.writeMessage(
                sessionMessage, to: clientToServerCharacteristic, peripheral: peripheral)
            self.log("Session establishment message written")

            if self.deviceResponseHandling == nil {
                DispatchQueue.main.asyncAfter(deadline: .now() + 1) { [weak self] in
                    self?.succeed("Session establishment message written successfully")
                }
            }
        }
    }

    func peripheral(
        _ peripheral: CBPeripheral,
        didUpdateNotificationStateFor characteristic: CBCharacteristic,
        error: Error?
    ) {
        if let error {
            fail(
                "Failed to subscribe to \(characteristic.uuid.uuidString): \(error.localizedDescription)"
            )
            return
        }

        if characteristic.uuid == serverToClientCharacteristicUuid, characteristic.isNotifying {
            log("Server-to-client notifications enabled")
        }
    }

    func peripheral(
        _ peripheral: CBPeripheral,
        didUpdateValueFor characteristic: CBCharacteristic,
        error: Error?
    ) {
        if let error {
            fail("Characteristic update failed: \(error.localizedDescription)")
            return
        }

        guard characteristic.uuid == serverToClientCharacteristicUuid else { return }
        guard let chunk = characteristic.value else {
            fail("Server-to-client characteristic update was missing data")
            return
        }

        serverToClientChunkCount += 1
        let prefix = chunk.first.map { String(format: "0x%02x", $0) } ?? "<missing>"
        log(
            "Received server-to-client chunk #\(serverToClientChunkCount) "
                + "length \(chunk.count) prefix \(prefix)"
        )

        handleDeviceResponseChunk(chunk, peripheral: peripheral)
    }

    private func handleDeviceResponseChunk(_ chunk: Data, peripheral: CBPeripheral) {
        guard !hasCompletedDeviceResponseFlow else { return }
        guard let deviceResponseHandling else {
            fail("Received an unexpected holder message")
            return
        }
        guard chunk.count >= 1 else {
            fail("Received an empty server-to-client chunk")
            return
        }

        deviceResponseBuffer.append(chunk.dropFirst())
        serverToClientBufferedByteCount = deviceResponseBuffer.count

        switch chunk[0] {
        case 0x00:
            let completeMessage = deviceResponseBuffer
            deviceResponseBuffer.removeAll(keepingCapacity: true)
            serverToClientBufferedByteCount = 0
            log(
                "Received final server-to-client chunk, complete encrypted message length \(completeMessage.count)"
            )

            do {
                let plaintext = try decryptedDeviceResponsePlaintext(
                    completeMessage, handling: deviceResponseHandling)
                if let expectedPlaintext = deviceResponseHandling.expectedPlaintext,
                    plaintext != expectedPlaintext
                {
                    throw ValidationError.invalidDeviceResponse(
                        "Reader received an unexpected device response payload")
                }
                if deviceResponseHandling.shouldPrintPlaintextHex {
                    print("\(Configuration.deviceResponseHexMarker)\(hexEncodedData(plaintext))")
                }
            } catch {
                fail("DeviceResponse validation failed: \(error.localizedDescription)")
                return
            }

            guard let stateCharacteristic else {
                fail("State characteristic became unavailable before BLE end")
                return
            }

            hasCompletedDeviceResponseFlow = true
            log("Validated encrypted device response")
            peripheral.writeValue(
                Data([Configuration.bleEndByte]),
                for: stateCharacteristic,
                type: .withoutResponse
            )
            log("Wrote BLE end byte")

            DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) { [weak self] in
                self?.succeed("Device response validated and BLE end sent successfully")
            }
        case 0x01:
            log(
                "Buffered intermediate server-to-client chunk, encrypted message length so far \(deviceResponseBuffer.count)"
            )
            break
        default:
            fail("Invalid first byte \(chunk[0]) in server-to-client data chunk")
        }
    }

    private func writeMessage(
        _ message: Data,
        to characteristic: CBCharacteristic,
        peripheral: CBPeripheral
    ) {
        let maxChunkSize = max(peripheral.maximumWriteValueLength(for: .withoutResponse) - 1, 1)
        var offset = 0

        while offset < message.count {
            let end = min(offset + maxChunkSize, message.count)
            var chunk = Data([end < message.count ? 0x01 : 0x00])
            chunk.append(message.subdata(in: offset..<end))
            peripheral.writeValue(chunk, for: characteristic, type: .withoutResponse)
            offset = end
        }
    }

    private func scan() {
        centralManager?.scanForPeripherals(
            withServices: [serviceUuid],
            options: [CBCentralManagerScanOptionAllowDuplicatesKey: false]
        )
    }

    private func restartScan() {
        guard let centralManager, centralManager.state == .poweredOn else { return }
        guard !hasTriggeredStart else { return }

        log("Restarting scan")
        scan()
    }

    private func succeed(_ message: String) {
        timeoutTimer?.invalidate()
        log(message)
        exit(0)
    }

    private func fail(_ message: String) {
        timeoutTimer?.invalidate()
        fputs("error: \(message)\n", stderr)
        exit(1)
    }

    private func log(_ message: String) {
        print("[CloseProximityMacReader] \(message)")
    }
}

do {
    let configuration = try parseConfiguration(arguments: Array(CommandLine.arguments.dropFirst()))
    let reader = CloseProximityReader(configuration: configuration)
    reader.run()
} catch {
    fputs("error: \(error.localizedDescription)\n", stderr)
    exit(2)
}
