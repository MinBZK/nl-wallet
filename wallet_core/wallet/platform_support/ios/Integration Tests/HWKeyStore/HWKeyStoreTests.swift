//
//  HWKeyStoreTests.swift
//  HWKeyStore Integration Tests
//
//  Created by Wallet Developer on 24/02/2023.
//

import Foundation
import Security

import PlatformSupport
import XCTest

final class HWKeyStoreTests: XCTestCase {
    static var platformSupport: PlatformSupport?
    static let identifier = "key"

    override class func setUp() {
        self.platformSupport = PlatformSupport.shared
    }

    override func tearDown() {
        let query: [String: Any] = [
            kSecClass as String: kSecClassKey,
            kSecAttrApplicationTag as String: Self.identifier.data(using: .utf8)!
        ]

        SecItemDelete(query as CFDictionary)
    }

    func testHardwareSignature() {
        // The Rust code will panic if this test fails.
        hw_keystore_test_hardware_signature()
    }

    func testHardwareEncryption() {
        // The Rust code will panic if this test fails.
        hw_keystore_test_hardware_encryption()
    }
}
