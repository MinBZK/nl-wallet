//
//  EncryptionKey.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 07/04/2023.
//

import Foundation

final class EncryptionKey {
    private static let identifierPrefix = "ecies"

    private func secureEnclaveKey(for identifier: String) throws -> SecureEnclaveKey {
        return try SecureEnclaveKey(identifier: "\(Self.identifierPrefix)_\(identifier)")
    }
}

extension EncryptionKey: EncryptionKeyBridge {
    func encrypt(identifier: String, payload: [UInt8]) throws -> [UInt8] {
        do {
            return try Array(self.secureEnclaveKey(for: identifier).encrypt(payload: Data(payload)))
        } catch let error as SecureEnclaveKeyError {
            throw KeyStoreError.from(error)
        }
    }

    func decrypt(identifier: String, payload: [UInt8]) throws -> [UInt8] {
        do {
            return try Array(self.secureEnclaveKey(for: identifier).decrypt(payload: Data(payload)))
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
