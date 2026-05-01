import Foundation

struct CloseProximityDisclosureActiveSessionState {
    let session: CloseProximityDisclosureActiveSession
    var connectionTask: Task<Void, Never>?
    var readMessagesTask: Task<Void, Never>?
    var sessionCrypto: CloseProximitySessionCrypto?
    var encodedSessionTranscript: [UInt8]?

    init(session: CloseProximityDisclosureActiveSession) {
        self.session = session
    }
}

struct CloseProximityDisclosureEstablishedSessionContext {
    let transport: CloseProximityBleTransport
    let sessionCrypto: CloseProximitySessionCrypto
}

struct CloseProximityDisclosureReaderSessionContext {
    let sessionCrypto: CloseProximitySessionCrypto
    let encodedSessionTranscript: [UInt8]
}

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

extension CloseProximityDisclosureActiveSessionState {
    var establishedSessionContext: CloseProximityDisclosureEstablishedSessionContext? {
        guard let sessionCrypto else { return nil }
        return CloseProximityDisclosureEstablishedSessionContext(
            transport: session.transport,
            sessionCrypto: sessionCrypto
        )
    }

    var readerSessionContext: CloseProximityDisclosureReaderSessionContext? {
        guard let sessionCrypto, let encodedSessionTranscript else { return nil }
        return CloseProximityDisclosureReaderSessionContext(
            sessionCrypto: sessionCrypto,
            encodedSessionTranscript: encodedSessionTranscript
        )
    }
}
