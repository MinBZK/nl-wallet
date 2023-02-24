//
//  KeyStore.swift
//  HWKeyStore
//
//  Created by Wallet Developer on 24/02/2023.
//

import Foundation

final class KeyStore {}

extension KeyStore: KeyStoreBridge {
    func getOrCreateKey(identifier: String) -> AsymmetricKeyBridge {
        return AsymmetricKey(key: SecureEnclaveKey(identifier: identifier))
    }
}
