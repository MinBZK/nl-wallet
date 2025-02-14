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

    func generate() async throws(AttestedKeyError) -> String {
        do {
            return try await Self.appAttest.generateKey()
        } catch let error as DCError {
            throw AttestedKeyError.from(error)
        } catch {
            fatalError(error.localizedDescription)
        }
    }

    func attest(identifier: String, challenge: [UInt8], googleCloudProjectId: UInt64) async throws(AttestedKeyError) -> AttestationData {
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
        // Retrieving the public key is only supported as part of key attestation for iOS.
        // This method is only implemented for Android and should not be called on this platform.
        throw .MethodUnimplemented
    }

    func delete(identifier _: String) throws(AttestedKeyError) {
        // Deleting an attested key is not supported by iOS.
        // This method is only implemented for Android and should not be called on this platform.
        throw .MethodUnimplemented
    }
}
