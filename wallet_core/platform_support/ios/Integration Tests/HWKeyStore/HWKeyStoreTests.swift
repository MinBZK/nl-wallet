//
//  HWKeyStoreTests.swift
//  HWKeyStore Integration Tests
//
//  Created by Wallet Developer on 24/02/2023.
//

import Foundation
import Security

import XCTest
import PlatformSupport

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
        XCTAssert(hw_keystore_test_hardware_signature())
    }

    func testHardwareEncryption() {
        XCTAssert(hw_keystore_test_hardware_encryption())
    }
}
