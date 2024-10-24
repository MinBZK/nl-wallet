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

    func generateIdentifier() throws(AttestedKeyError) -> String {
        do {
            return try AppAttest.generateKey()
        } catch {
            throw AttestedKeyError.from(error)
        }
    }

    func attest(identifier: String, challenge: [UInt8]) throws(IdentifierAttestedKeyError) -> AttestationData {
        do {
            let attestation = try AppAttest.attestKey(keyId: identifier, clientDataHash: Data(challenge))

            return .apple(attestationData: Array(attestation))
        } catch {
            throw IdentifierAttestedKeyError.from(error)
        }
    }

    func sign(identifier: String, payload: [UInt8]) throws(AttestedKeyError) -> [UInt8] {
        do {
            let assertion = try AppAttest.generateAssertion(keyId: identifier, clientData: Data(payload))

            return Array(assertion)
        } catch {
            throw AttestedKeyError.from(error)
        }
    }

    func publicKey(identifier: String) throws(AttestedKeyError) -> [UInt8] {
        throw AttestedKeyError.KeyError(reason: "not supported on this platform")
    }

    func delete(identifier: String) throws(AttestedKeyError) {
        throw AttestedKeyError.KeyError(reason: "not supported on this platform")
    }
}
