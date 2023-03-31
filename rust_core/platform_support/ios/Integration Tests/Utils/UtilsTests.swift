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
    private static var utils: Utils?

    override class func setUp() {
        self.utils = Utils.shared
    }

    func testAllUtilities() {
        // TODO: implement integration tests
        XCTAssert(true)
    }
}
