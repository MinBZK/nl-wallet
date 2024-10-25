//
//  AppAttest.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 24/10/2024.
//

import CryptoKit
import DeviceCheck
import Foundation

enum AppAttest {
    private static let queue = DispatchQueue(label: String(describing: Self.self), qos: .userInitiated)

    static func generateKey() throws(AppAttestError) -> String {
        let appAttest = DCAppAttestService.shared
        guard appAttest.isSupported else {
            throw .unsupported
        }

        let result: Result<String, AppAttestError> = self.queue.sync {
            let semaphore = DispatchSemaphore(value: 0)
            var keyId: String?
            var error: Error?

            appAttest.generateKey { cbKeyId, cbError in
                keyId = cbKeyId
                error = cbError

                semaphore.signal()
            }

            semaphore.wait()

            guard let keyId else {
                return .failure(.generate(error: error))
            }

            return .success(keyId)
        }

        return try result.get()
    }

    static func attestKey(keyId: String, clientDataHash: Data) throws(AppAttestError) -> Data {
        let appAttest = DCAppAttestService.shared
        guard appAttest.isSupported else {
            throw .unsupported
        }

        let result: Result<Data, AppAttestError> = self.queue.sync {
            let semaphore = DispatchSemaphore(value: 0)
            var attestation: Data?
            var error: Error?

            appAttest.attestKey(keyId, clientDataHash: clientDataHash) { cbAttestation, cbError in
                attestation = cbAttestation
                error = cbError

                semaphore.signal()
            }

            semaphore.wait()

            guard let attestation else {
                return .failure(.attest(error: error))
            }

            return .success(attestation)
        }

        return try result.get()
    }

    static func generateAssertion(keyId: String, clientData: Data) throws(AppAttestError) -> Data {
        let appAttest = DCAppAttestService.shared
        guard appAttest.isSupported else {
            throw .unsupported
        }

        let clientDataHash = Data(SHA256.hash(data: clientData))

        let result: Result<Data, AppAttestError> = self.queue.sync {
            let semaphore = DispatchSemaphore(value: 0)
            var assertion: Data?
            var error: Error?

            appAttest.generateAssertion(keyId, clientDataHash: clientDataHash) { cbAssertion, cbError in
                assertion = cbAssertion
                error = cbError

                semaphore.signal()
            }

            semaphore.wait()

            guard let assertion else {
                return .failure(.assert(error: error))
            }

            return .success(assertion)
        }

        return try result.get()
    }
}
