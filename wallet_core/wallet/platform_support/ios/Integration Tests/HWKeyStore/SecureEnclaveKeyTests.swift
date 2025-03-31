//
//  SecureEnclaveKeyTests.swift
//  Integration Tests
//
//  Created by Wallet Developer on 24/02/2023.
//

import Foundation
import Security

@testable import PlatformSupport
import XCTest

final class SecureEnclaveKeyTests: XCTestCase {
    static let identifiers = ["key_identifier1", "key_identifier2"]

    private static func errorMessage(for unmanagedError: Unmanaged<CFError>?) -> String {
        guard let unmanagedError else {
            return "Unknown error"
        }

        let error = unmanagedError.takeRetainedValue()

        return error.localizedDescription
    }

    private static func publicKey(for key: borrowing SecureEnclaveKey) -> SecKey {
        var error: Unmanaged<CFError>?
        let publicKeyAttributes: [String: Any] = [
            kSecAttrKeyType as String: kSecAttrKeyTypeEC,
            kSecAttrKeyClass as String: kSecAttrKeyClassPublic
        ]

        let der = try! key.encodePublicKey()[26...]
        let publicKey = SecKeyCreateWithData(der as CFData, publicKeyAttributes as CFDictionary, &error)

        guard let publicKey else {
            preconditionFailure("Could not decode public key: \(Self.errorMessage(for: error))")
        }

        return publicKey
    }

    private static func isValid(signature: Data, for payload: Data, with key: borrowing SecureEnclaveKey) -> Bool {
        var error: Unmanaged<CFError>?
        let publicKey = self.publicKey(for: key)

        let isValid = SecKeyVerifySignature(publicKey, .ecdsaSignatureMessageX962SHA256, payload as CFData, signature as CFData, &error)

        if let error, CFErrorGetCode(error.takeRetainedValue()) != errSecVerifyFailed {
            preconditionFailure("Could not verify signature: \(Self.errorMessage(for: error))")
        }

        return isValid
    }

    override func tearDown() {
        for identifier in Self.identifiers {
            let query: [String: Any] = [
                kSecClass as String: kSecClassKey,
                kSecAttrApplicationTag as String: identifier.data(using: .utf8)!
            ]

            SecItemDelete(query as CFDictionary)
        }
    }

    func testPublicKey() {
        let key1 = try! SecureEnclaveKey(identifier: Self.identifiers[0])
        let key1Again = try! SecureEnclaveKey(identifier: Self.identifiers[0])
        let key2 = try! SecureEnclaveKey(identifier: Self.identifiers[1])

        XCTAssertGreaterThan(try! key1.encodePublicKey().count, 0)
        XCTAssertEqual(try! key1.encodePublicKey(),
                       try! key1.encodePublicKey(),
                       "Public keys from the same source should be equal")
        XCTAssertEqual(try! key1.encodePublicKey(),
                       try! key1Again.encodePublicKey(),
                       "Public keys for the same identifier should be equal")
        XCTAssertNotEqual(try! key1.encodePublicKey(),
                          try! key2.encodePublicKey(),
                          "Public keys for different identifiers should differ")

        let _ = Self.publicKey(for: key1)
        let _ = Self.publicKey(for: key2)
    }

    func testSign() {
        let message = "This is a message that will be signed.".data(using: .ascii)!

        let key1 = try! SecureEnclaveKey(identifier: Self.identifiers[0])
        let key1Again = try! SecureEnclaveKey(identifier: Self.identifiers[0])
        let key2 = try! SecureEnclaveKey(identifier: Self.identifiers[1])

        let emptySignature = try! key1.sign(payload: Data())
        let signature1 = try! key1.sign(payload: message)
        let signature1Repeat = try! key1.sign(payload: message)
        let signature1Again = try! key1Again.sign(payload: message)
        let signature2 = try! key2.sign(payload: message)

        XCTAssertGreaterThan(emptySignature.count, 0, "An empty payload should produce a signature")
        XCTAssertNotEqual(signature1, signature1Repeat, "Signatures signed with the same key instance should differ")
        XCTAssertNotEqual(signature1, signature1Again, "Signatures signed with the same key should differ")
        XCTAssertNotEqual(signature1, signature2, "Signatures signed with a different key should differ")

        XCTAssertTrue(Self.isValid(signature: signature1, for: message, with: key1), "Signature should be valid")
        XCTAssertTrue(Self.isValid(signature: signature1Repeat, for: message, with: key1), "Signature should be valid")
        XCTAssertTrue(Self.isValid(signature: signature1Again, for: message, with: key1), "Signature should be valid")
        XCTAssertTrue(Self.isValid(signature: signature2, for: message, with: key2), "Signature should be valid")
    }

    func testEncrytAndDecrypt() {
        let message = "This is a message that will be encrypted and then decrypted.".data(using: .ascii)!

        let key1 = try! SecureEnclaveKey(identifier: Self.identifiers[0])
        let key2 = try! SecureEnclaveKey(identifier: Self.identifiers[1])

        let encrypted1 = try! key1.encrypt(payload: message)
        let encrypted2 = try! key2.encrypt(payload: message)
        let decrypted1 = try! key1.decrypt(payload: encrypted1)
        let decrypted2 = try! key2.decrypt(payload: encrypted2)

        XCTAssertGreaterThan(encrypted1.count, 0, "An encrypted payload should not be empty")
        XCTAssertGreaterThan(encrypted2.count, 0, "An encrypted payload should not be empty")
        XCTAssertNotEqual(encrypted1, message, "An encrypted payload should differ from its source")
        XCTAssertNotEqual(encrypted2, message, "An encrypted payload should differ from its source")
        XCTAssertNotEqual(encrypted1, encrypted2, "Payloads encrypted with different keys should differ")
        XCTAssertEqual(decrypted1, message, "A decrypted payload should equal its source")
        XCTAssertEqual(decrypted2, message, "A decrypted payload should equal its source")
    }

    func testDelete() {
        let message = "This is a message that will be both signed and encrypted.".data(using: .ascii)!

        // Create a new private key.
        let key1 = try! SecureEnclaveKey(identifier: Self.identifiers[0])
        let signature1 = try! key1.sign(payload: message)
        let encrypted1 = try! key1.encrypt(payload: message)

        // Create a new key with the same identifier, which should be backed by the same private key.
        let key2 = try! SecureEnclaveKey(identifier: Self.identifiers[0])
        let signature2 = try! key2.sign(payload: message)
        let encrypted2 = try! key2.encrypt(payload: message)

        XCTAssertTrue(Self.isValid(signature: signature1, for: message, with: key1), "The signature should be valid for the key itself")
        XCTAssertTrue(Self.isValid(signature: signature2, for: message, with: key1), "The signature should be valid for the second copy of the key")

        XCTAssertEqual(try! key1.decrypt(payload: encrypted1), message, "The message should decrypt for the key itself")
        XCTAssertEqual(try! key1.decrypt(payload: encrypted2), message, "The message should decrypt for the second copy of the key")

        // Now delete this private key, the second call should be a no-op.
        try! key1.delete()
        try! key2.delete()

        // Create a key with the same identifier, which should result in a different key.
        let key3 = try! SecureEnclaveKey(identifier: Self.identifiers[0])
        let signature3 = try! key3.sign(payload: message)
        let encrypted3 = try! key3.encrypt(payload: message)

        XCTAssertFalse(Self.isValid(signature: signature1, for: message, with: key3), "The signature should not be valid for the new key")
        XCTAssertFalse(Self.isValid(signature: signature2, for: message, with: key3), "The signature should not be valid for the new key")
        XCTAssertTrue(Self.isValid(signature: signature3, for: message, with: key3), "The signature should be valid for the new key itself")

        XCTAssertThrowsError(try key3.decrypt(payload: encrypted1), "Decrypting the message should fail with the new key")
        XCTAssertThrowsError(try key3.decrypt(payload: encrypted2), "Decrypting the message should fail with the new key")
        XCTAssertEqual(try! key3.decrypt(payload: encrypted3), message, "The message should decrypt for the new key itself")
    }
}
