//
//  PlatformUtilityTests.swift
//  Integration Tests
//
//  Created by The Wallet Developers on 31/03/2023.
//

import Foundation

@testable import PlatformSupport
import XCTest

final class PlatformUtilityTests: XCTestCase {
    func testUrlForAppSupportDirectory() throws {
        let url = try PlatformUtility.urlForAppStorageWithoutBackup()

        XCTAssert(url.isFileURL, "URL should be a file URL")
        XCTAssertGreaterThan(url.path.count, 0, "URL path should not be an empty string")
    }
}
