//
//  HWKeyStoreTests.swift
//  HWKeyStore Integration Tests
//
//  Created by Wallet Developer on 24/02/2023.
//

import Foundation
import Security

import XCTest
import HWKeyStore

final class HWKeyStoreTests: XCTestCase {
    static var keyStore: HWKeyStore?

    override class func setUp() {
        self.keyStore = HWKeyStore.shared
    }

    func testHardwareSignature() {
        XCTAssert(test_hardware_signature())
    }
}
