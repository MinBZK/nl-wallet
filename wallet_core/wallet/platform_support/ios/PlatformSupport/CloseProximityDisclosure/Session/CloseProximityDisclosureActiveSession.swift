import Foundation
@preconcurrency import Multipaz

struct CloseProximityDisclosureActiveSessionState {
    let session: CloseProximityDisclosureActiveSession
    var connectionTask: Task<Void, Never>?
    var readMessagesTask: Task<Void, Never>?
    var sessionEncryption: SessionEncryption?
    var encodedSessionTranscript: KotlinByteArray?

    init(session: CloseProximityDisclosureActiveSession) {
        self.session = session
    }
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
    let channel: CloseProximityDisclosureChannel
    let transport: CloseProximityBleTransport
    let eDeviceKey: EcPrivateKey
    let encodedDeviceEngagement: KotlinByteArray

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
}

extension CloseProximityDisclosureActiveSessionState {
    var establishedSessionContext: CloseProximityDisclosureEstablishedSessionContext? {
        guard let sessionEncryption else { return nil }
        return CloseProximityDisclosureEstablishedSessionContext(
            transport: session.transport,
            sessionEncryption: sessionEncryption
        )
    }

    var readerSessionContext: CloseProximityDisclosureReaderSessionContext? {
        guard let sessionEncryption, let encodedSessionTranscript else { return nil }
        return CloseProximityDisclosureReaderSessionContext(
            sessionEncryption: sessionEncryption,
            encodedSessionTranscript: encodedSessionTranscript
        )
    }
}
