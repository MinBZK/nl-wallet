//
//  Iso180135Tests.swift
//  Integration Tests
//
//  Created by The Wallet Developers on 06/03/2026.
//

import Foundation

import PlatformSupport
import XCTest

final class Iso180135Tests: XCTestCase {
    static var platformSupport: PlatformSupport?

    override class func setUp() {
        self.platformSupport = PlatformSupport.shared
    }

    func testAllIso180135s() {
        // The Rust code will panic if this test fails.
        iso18013_5_test_start_qr_handover()
    }
}
