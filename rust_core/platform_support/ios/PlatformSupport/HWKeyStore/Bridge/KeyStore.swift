//
//  KeyStore.swift
//  HWKeyStore
//
//  Created by Wallet Developer on 24/02/2023.
//

import Foundation

final class KeyStore {}

extension KeyStore: KeyStoreBridge {
    func getOrCreateSigningKey(identifier: String) throws -> SigningKeyBridge {
        do {
            return try SigningKey(key: SecureEnclaveKey(identifier: identifier))
        } catch let error as SecureEnclaveKeyError {
            throw KeyStoreError.from(error)
        }
    }
    
    func getOrCreateEncryptionKey(identifier: String) throws -> EncryptionKeyBridge {
        //TODO: Implement getOrCreateEncryptionKey
        fatalError("Not yet implemented")
    }
}
