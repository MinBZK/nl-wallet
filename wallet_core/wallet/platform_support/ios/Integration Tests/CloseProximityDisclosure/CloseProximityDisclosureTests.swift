//
//  CloseProximityDisclosureTests.swift
//  Integration Tests
//
//  Created by The Wallet Developers on 06/03/2026.
//

import Foundation

import PlatformSupport
import XCTest

final class CloseProximityDisclosureTests: XCTestCase {
    static var platformSupport: PlatformSupport?

    override class func setUp() {
        self.platformSupport = PlatformSupport.shared
    }

    func testAllCloseProximityDisclosures() {
        // The Rust code will panic if this test fails.
        close_proximity_disclosure_test_start_qr_handover()
    }
}
