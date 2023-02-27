//
//  AsymmetricKey.swift
//  HWKeyStore
//
//  Created by Wallet Developer on 24/02/2023.
//

import Foundation

final class AsymmetricKey {
    let key: SecureEnclaveKey

    init(key: SecureEnclaveKey) {
        self.key = key
    }
}

extension AsymmetricKey: AsymmetricKeyBridge {
    func publicKey() throws -> [UInt8] {
        do {
            return Array(try self.key.publicKey())
        } catch let error as SecureEnclaveKeyError {
            throw KeyStoreError.from(error)
        }
    }

    func sign(payload: [UInt8]) throws -> [UInt8] {
        do {
            return Array(try self.key.sign(payload: Data(payload)))
        } catch let error as SecureEnclaveKeyError {
            throw KeyStoreError.from(error)
        }
    }
}
