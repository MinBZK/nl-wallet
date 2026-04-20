//
//  AttestedKeyTests.swift
//  Integration Tests
//
//  Created by The Wallet Developers on 28/10/2024.
//

import Foundation

import PlatformSupport
import XCTest

final class AttestedKeyTests: XCTestCase {
    static var platformSupport: PlatformSupport?

    override class func setUp() {
        self.platformSupport = PlatformSupport.shared
    }

    func testHardwareSignature() {
        // The Rust code will panic if this test fails.
        ios_attested_key_test()
    }
}
