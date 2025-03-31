//
//  SigningKey.swift
//  HWKeyStore
//
//  Created by Wallet Developer on 24/02/2023.
//

import Foundation

final class SigningKey {
    private static let identifierPrefix = "ecdsa"

    private func secureEnclaveKey(for identifier: String) throws -> SecureEnclaveKey {
        return try SecureEnclaveKey(identifier: "\(Self.identifierPrefix)_\(identifier)")
    }
}

extension SigningKey: SigningKeyBridge {
    func publicKey(identifier: String) throws -> [UInt8] {
        do {
            return try Array(self.secureEnclaveKey(for: identifier).encodePublicKey())
        } catch let error as SecureEnclaveKeyError {
            throw KeyStoreError.from(error)
        }
    }

    func sign(identifier: String, payload: [UInt8]) throws -> [UInt8] {
        do {
            return try Array(self.secureEnclaveKey(for: identifier).sign(payload: Data(payload)))
        } catch let error as SecureEnclaveKeyError {
            throw KeyStoreError.from(error)
        }
    }

    func delete(identifier: String) throws {
        do {
            return try self.secureEnclaveKey(for: identifier).delete()
        } catch let error as SecureEnclaveKeyError {
            throw KeyStoreError.from(error)
        }
    }
}
