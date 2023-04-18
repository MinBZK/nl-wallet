//
//  SigningKey.swift
//  HWKeyStore
//
//  Created by Wallet Developer on 24/02/2023.
//

import Foundation

final class SigningKey {
    let key: SecureEnclaveKey

    init(key: SecureEnclaveKey) {
        self.key = key
    }
}

extension SigningKey: SigningKeyBridge {
    func publicKey() throws -> [UInt8] {
        do {
            return try Array(self.key.encodePublicKey())
        } catch let error as SecureEnclaveKeyError {
            throw KeyStoreError.from(error)
        }
    }

    func sign(payload: [UInt8]) throws -> [UInt8] {
        do {
            return try Array(self.key.sign(payload: Data(payload)))
        } catch let error as SecureEnclaveKeyError {
            throw KeyStoreError.from(error)
        }
    }
}
