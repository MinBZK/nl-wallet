//
//  UtilsTests.swift
//  Integration Tests
//
//  Created by The Wallet Developers on 31/03/2023.
//

import Foundation

import XCTest
import PlatformSupport

final class UtilsTests: XCTestCase {
    static var platformSupport: PlatformSupport?

    override class func setUp() {
        self.platformSupport = PlatformSupport.shared
    }

    func testAllUtilities() {
        // The Rust code will panic if this test fails.
        utils_test_get_storage_path()
    }
}
