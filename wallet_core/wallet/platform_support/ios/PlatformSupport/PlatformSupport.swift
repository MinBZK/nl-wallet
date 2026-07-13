//
//  PlatformSupport.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 13/04/2023.
//

import Foundation

public final class PlatformSupport {
    public static let shared = PlatformSupport()

    public var allowReleaseLogs: Bool {
        get { PlatformSupportLogging.allowReleaseLogs }
        set { PlatformSupportLogging.allowReleaseLogs = newValue }
    }

    private let signingKey: SigningKey
    private let encryptionKey: EncryptionKey
    private let attestedKey: AttestedKey
    private let utilities: Utilities
    private let closeProximityDisclosure: CloseProximityDisclosure

    private init() {
        self.signingKey = SigningKey()
        self.encryptionKey = EncryptionKey()
        self.attestedKey = AttestedKey()
        self.utilities = Utilities()
        self.closeProximityDisclosure = CloseProximityDisclosure()

        initPlatformSupport(
            signingKey: self.signingKey,
            encryptionKey: self.encryptionKey,
            attestedKey: self.attestedKey,
            utils: self.utilities,
            closeProximityDisclosure: self.closeProximityDisclosure
        )
    }
}

enum PlatformSupportLogging {
    static var allowReleaseLogs = false

    static var allowLogs: Bool {
        #if DEBUG
        return true
        #else
        return allowReleaseLogs
        #endif
    }
}
