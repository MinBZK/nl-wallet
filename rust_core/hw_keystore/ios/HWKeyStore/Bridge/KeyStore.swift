//
//  KeyStore.swift
//  HWKeyStore
//
//  Created by Wallet Developer on 24/02/2023.
//

import Foundation

final class KeyStore {}

extension KeyStore: KeyStoreBridge {
    func getOrCreateKey(identifier: String) throws -> AsymmetricKeyBridge {
        do {
            return AsymmetricKey(key: try SecureEnclaveKey(identifier: identifier))
        } catch let error as SecureEnclaveKeyError {
            throw KeyStoreError.from(error)
        }
    }
}
