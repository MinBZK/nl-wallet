//
//  PlatformSupport.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 13/04/2023.
//

import Foundation

public final class PlatformSupport {
    public static let shared = PlatformSupport()

    private let keystore: KeyStore
    private let utilities: Utilities

    private init() {
        self.keystore = KeyStore()
        self.utilities = Utilities()

        initPlatformSupport(keyStore: self.keystore, utils: self.utilities)
    }
}
