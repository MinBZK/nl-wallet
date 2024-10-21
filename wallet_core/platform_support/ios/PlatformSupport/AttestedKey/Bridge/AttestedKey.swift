//
//  AttestedKey.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 24/10/2024.
//

import Foundation

final class AttestedKey {}

extension AttestedKey: AttestedKeyBridge {
    func keyType() throws -> AttestedKeyType {
        .apple
    }

    func generateIdentifier() throws -> String {
        throw AttestedKeyError.KeyError(reason: "unimplemented")
    }

    func attest(identifier: String, challenge: [UInt8]) throws -> AttestationData {
        throw AttestedKeyError.KeyError(reason: "unimplemented")
    }

    func sign(identifier: String, payload: [UInt8]) throws -> [UInt8] {
        throw AttestedKeyError.KeyError(reason: "unimplemented")
    }

    func publicKey(identifier: String) throws -> [UInt8] {
        throw AttestedKeyError.KeyError(reason: "not supported on this platform")
    }

    func delete(identifier: String) throws {
        throw AttestedKeyError.KeyError(reason: "not supported on this platform")
    }
}
