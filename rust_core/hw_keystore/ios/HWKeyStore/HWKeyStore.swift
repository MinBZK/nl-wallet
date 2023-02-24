//
//  HWKeyStore.swift
//  HWKeyStore
//
//  Created by Wallet Developer on 24/02/2023.
//

public final class HWKeyStore {
    public static let shared = HWKeyStore()

    private let keystore: KeyStore

    private init() {
        self.keystore = KeyStore()

        initHwKeystore(bridge: self.keystore)
    }
}
