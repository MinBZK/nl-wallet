//
//  KeyStore.swift
//  HWKeyStore
//
//  Created by Wallet Developer on 24/02/2023.
//

import Foundation

final class KeyStore {
    private static let signingPrefix = "ecdsa"
    private static let encryptionPrefix = "ecies"
}

extension KeyStore: KeyStoreBridge {
    func getOrCreateSigningKey(identifier: String) throws -> SigningKeyBridge {
        let identifier = "\(Self.signingPrefix)_\(identifier)"

        do {
            return try SigningKey(key: SecureEnclaveKey(identifier: identifier))
        } catch let error as SecureEnclaveKeyError {
            throw KeyStoreError.from(error)
        }
    }

    func getOrCreateEncryptionKey(identifier: String) throws -> EncryptionKeyBridge {
        let identifier = "\(Self.encryptionPrefix)_\(identifier)"

        do {
            return try EncryptionKey(key: SecureEnclaveKey(identifier: identifier))
        } catch let error as SecureEnclaveKeyError {
            throw KeyStoreError.from(error)
        }
    }
}
