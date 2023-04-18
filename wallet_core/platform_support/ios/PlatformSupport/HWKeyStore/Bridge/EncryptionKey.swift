//
//  EncryptionKey.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 07/04/2023.
//

import Foundation

final class EncryptionKey {
    let key: SecureEnclaveKey

    init(key: SecureEnclaveKey) {
        self.key = key
    }
}

extension EncryptionKey: EncryptionKeyBridge {
    func encrypt(payload: [UInt8]) throws -> [UInt8] {
        do {
            return try Array(self.key.encrypt(payload: Data(payload)))
        } catch let error as SecureEnclaveKeyError {
            throw KeyStoreError.from(error)
        }
    }

    func decrypt(payload: [UInt8]) throws -> [UInt8] {
        do {
            return try Array(self.key.decrypt(payload: Data(payload)))
        } catch let error as SecureEnclaveKeyError {
            throw KeyStoreError.from(error)
        }
    }
}
