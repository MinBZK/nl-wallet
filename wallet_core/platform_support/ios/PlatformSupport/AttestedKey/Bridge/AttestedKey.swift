//
//  AttestedKey.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 24/10/2024.
//

import CryptoKit
import DeviceCheck
import Foundation

final class AttestedKey {
    private static let appAttest = DCAppAttestService.shared
}

extension AttestedKey: AttestedKeyBridge {
    func keyType() -> AttestedKeyType {
        .apple
    }

    func generateIdentifier() async throws(AttestedKeyError) -> String {
        do {
            return try await Self.appAttest.generateKey()
        } catch let error as DCError {
            throw AttestedKeyError.from(error)
        } catch {
            fatalError(error.localizedDescription)
        }
    }

    func attest(identifier: String, challenge: [UInt8]) async throws(AttestedKeyError) -> AttestationData {
        do {
            let attestation = try await Self.appAttest.attestKey(identifier, clientDataHash: Data(challenge))

            return .apple(attestationData: Array(attestation))
        } catch let error as DCError {
            throw AttestedKeyError.from(error)
        } catch {
            fatalError(error.localizedDescription)
        }
    }

    func sign(identifier: String, payload: [UInt8]) async throws(AttestedKeyError) -> [UInt8] {
        let clientDataHash = Data(SHA256.hash(data: Data(payload)))

        do {
            let assertion = try await Self.appAttest.generateAssertion(identifier, clientDataHash: clientDataHash)
            return Array(assertion)
        } catch let error as DCError {
            throw AttestedKeyError.from(error)
        } catch {
            fatalError(error.localizedDescription)
        }
    }

    func publicKey(identifier _: String) throws(AttestedKeyError) -> [UInt8] {
        throw .MethodUnimplemented
    }

    func delete(identifier _: String) throws(AttestedKeyError) {
        throw .MethodUnimplemented
    }
}
