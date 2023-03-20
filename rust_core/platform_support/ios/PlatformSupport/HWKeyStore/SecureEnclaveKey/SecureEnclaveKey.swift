//
//  SecureEnclaveKey.swift
//  HWKeyStore
//
//  Created by Wallet Developer on 24/02/2023.
//

import Foundation
import Security

final class SecureEnclaveKey {
    // MARK: - Static properties

    // We want to return a key in PKIX, ASN.1 DER form, but SecKeyCopyExternalRepresentation
    // returns the coordinates X and Y of the public key as follows: 04 || X || Y. We convert
    // that to a valid PKIX key by prepending the SPKI of secp256r1 in DER format.
    // Based on https://stackoverflow.com/a/45188232
    private static let secp256r1Header = Data([
        0x30, 0x59, 0x30, 0x13, 0x06, 0x07, 0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x02, 0x01,
        0x06, 0x08, 0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x03, 0x01, 0x07, 0x03, 0x42, 0x00
    ])

    // MARK: - Static methods

    private static func tag(from identifier: String) -> Data {
        return identifier.data(using: .utf8)!
    }

    private static func error(for unmanagedError: Unmanaged<CFError>?) -> Error? {
        guard let unmanagedError else {
            return nil
        }

        let error = unmanagedError.takeRetainedValue()

        return error
    }

    private static func fetchKey(with identifier: String) throws -> SecKey? {
        let query: [String: Any] = [
            kSecClass as String: kSecClassKey,
            kSecAttrTokenID as String: kSecAttrTokenIDSecureEnclave,
            kSecAttrApplicationTag as String: self.tag(from: identifier),
            kSecAttrKeyType as String: kSecAttrKeyTypeEC,
            kSecReturnRef as String: true
        ]

        var item: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &item)

        switch status {
        case errSecSuccess:
            break
        case errSecItemNotFound:
            return nil
        default:
            let errorMessage: String? = {
                guard #available(iOS 11.3, *),
                      let errorMessage = SecCopyErrorMessageString(status, nil) else {
                    return nil
                }

                return errorMessage as String
            }()

            throw SecureEnclaveKeyError.fetch(errorMessage: errorMessage)
        }

        return (item as! SecKey)
    }

    private static func createKey(with identifier: String) throws -> SecKey {
        var error: Unmanaged<CFError>?

        guard let access = SecAccessControlCreateWithFlags(
            kCFAllocatorDefault,
            kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
            .privateKeyUsage,
            &error
        ) else {
            throw SecureEnclaveKeyError.create(keyChainError: self.error(for: error))
        }

        let keyAttributes: [String: Any] = [
            kSecAttrIsPermanent as String: true,
            kSecAttrApplicationTag as String: self.tag(from: identifier),
            kSecAttrAccessControl as String: access
        ]
        let attributes: [String: Any] = [
            kSecAttrKeyType as String: kSecAttrKeyTypeEC,
            kSecAttrKeySizeInBits as String: 256,
            kSecAttrTokenID as String: kSecAttrTokenIDSecureEnclave,
            kSecPrivateKeyAttrs as String: keyAttributes
        ]

        guard let key = SecKeyCreateRandomKey(attributes as CFDictionary, &error) else {
            throw SecureEnclaveKeyError.create(keyChainError: self.error(for: error))
        }

        return key
    }

    private static func derivePublicKey(from privateKey: SecKey) throws -> Data {
        guard let publicKey = SecKeyCopyPublicKey(privateKey) else {
            fatalError("Error while deriving public key")
        }

        var error: Unmanaged<CFError>?
        guard let keyData = SecKeyCopyExternalRepresentation(publicKey, &error) else {
            throw SecureEnclaveKeyError.derivePublicKey(keyChainError: self.error(for: error))
        }

        return self.secp256r1Header + (keyData as Data)
    }

    private static func sign(payload: Data, with privateKey: SecKey) throws -> Data {
        var error: Unmanaged<CFError>?
        guard let signature = SecKeyCreateSignature(privateKey,
                                                    .ecdsaSignatureMessageX962SHA256,
                                                    payload as CFData,
                                                    &error) else {
            throw SecureEnclaveKeyError.sign(keyChainError: self.error(for: error))
        }

        return signature as Data
    }

    // MARK: - Instance properties

    let identifier: String

    private let privateKey: SecKey

    // MARK: - Initializer

    init(identifier: String) throws {
        self.identifier = identifier

        self.privateKey = try {
            guard let privateKey = try Self.fetchKey(with: identifier) else {
                return try Self.createKey(with: identifier)
            }

            return privateKey
        }()
    }

    // MARK: - Instance methods

    func publicKey() throws -> Data {
        return try Self.derivePublicKey(from: self.privateKey)
    }

    func sign(payload: Data) throws -> Data {
        return try Self.sign(payload: payload, with: self.privateKey)
    }
}
