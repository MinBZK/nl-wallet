//
//  PlatformSupport.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 13/04/2023.
//

import Foundation

public final class PlatformSupport {
    public static let shared = PlatformSupport()

    private let signingKey: SigningKey
    private let encryptionKey: EncryptionKey
    private let attestedKey: AttestedKey
    private let utilities: Utilities

    private init() {
        self.signingKey = SigningKey()
        self.encryptionKey = EncryptionKey()
        self.attestedKey = AttestedKey()
        self.utilities = Utilities()

        initPlatformSupport(
            signingKey: self.signingKey,
            encryptionKey: self.encryptionKey,
            attestedKey: self.attestedKey,
            utils: self.utilities
        )
    }
}
