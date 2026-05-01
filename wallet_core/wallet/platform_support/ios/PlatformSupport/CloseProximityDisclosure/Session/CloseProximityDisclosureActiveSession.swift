import Foundation

enum CloseProximitySessionStatusCode {
    static let sessionEncryptionError: Int64 = 10
    static let cborDecodingError: Int64 = 11
    static let termination: Int64 = 20
}

final class CloseProximityDisclosureActiveSession {
    let channel: CloseProximityDisclosureChannel
    let transport: CloseProximityBleTransport
    let eDevicePrivateKey: [UInt8]
    let encodedDeviceEngagement: [UInt8]
    // Actor-owned mutable state populated while the session is active.
    var connectionTask: Task<Void, Never>?
    var sessionCrypto: CloseProximitySessionCrypto?
    var encodedSessionTranscript: [UInt8]?

    init(
        channel: CloseProximityDisclosureChannel,
        transport: CloseProximityBleTransport,
        eDevicePrivateKey: [UInt8],
        encodedDeviceEngagement: [UInt8]
    ) {
        self.channel = channel
        self.transport = transport
        self.eDevicePrivateKey = eDevicePrivateKey
        self.encodedDeviceEngagement = encodedDeviceEngagement
    }
}
