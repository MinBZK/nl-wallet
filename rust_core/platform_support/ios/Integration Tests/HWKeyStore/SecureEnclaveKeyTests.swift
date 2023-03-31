//
//  SecureEnclaveKeyTests.swift
//  Integration Tests
//
//  Created by Wallet Developer on 24/02/2023.
//

import Foundation
import Security

import XCTest
@testable import PlatformSupport

final class SecureEnclaveKeyTests: XCTestCase {
    static let identifiers = ["key_identifier1", "key_identifier2"]

    private static func errorMessage(for unmanagedError: Unmanaged<CFError>?) -> String {
        guard let unmanagedError else {
            return "Unknown error"
        }

        let error = unmanagedError.takeRetainedValue()

        return error.localizedDescription
    }

    private static func publicKey(for key: SecureEnclaveKey) -> SecKey {
        var error: Unmanaged<CFError>?
        let publicKeyAttributes: [String: Any] = [
            kSecAttrKeyType as String: kSecAttrKeyTypeEC,
            kSecAttrKeyClass as String: kSecAttrKeyClassPublic
        ]

        let der = try! key.publicKey()[26...]
        let publicKey = SecKeyCreateWithData(der as CFData, publicKeyAttributes as CFDictionary, &error)

        guard let publicKey else {
            preconditionFailure("Could not decode public key: \(Self.errorMessage(for: error))")
        }

        return publicKey
    }

    private static func isValid(signature: Data, for payload: Data, with key: SecureEnclaveKey) -> Bool {
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

    func testInit() {
        // first instance should create a key for the identifier
        let key1 = try! SecureEnclaveKey(identifier: Self.identifiers[0])
        // second instance should retrieve the newly created key with the identifier
        let key1Again = try! SecureEnclaveKey(identifier: Self.identifiers[0])

        XCTAssert(key1 !== key1Again)
    }

    func testPublicKey() {
        let key1 = try! SecureEnclaveKey(identifier: Self.identifiers[0])
        let key1Again = try! SecureEnclaveKey(identifier: Self.identifiers[0])
        let key2 = try! SecureEnclaveKey(identifier: Self.identifiers[1])

        XCTAssertGreaterThan(try! key1.publicKey().count, 0)
        XCTAssertEqual(try! key1.publicKey(),
                       try! key1.publicKey(),
                       "Public keys from the same source should be equal")
        XCTAssertEqual(try! key1.publicKey(),
                       try! key1Again.publicKey(),
                       "Public keys for the same identifier should be equal")
        XCTAssertNotEqual(try! key1.publicKey(),
                          try! key2.publicKey(),
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
}
