//
//  Utilities.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 31/03/2023.
//

import Foundation

final class Utilities {}

extension Utilities: UtilitiesBridge {
    func getStoragePath() throws -> String {
        do {
            let url = try PlatformUtility.urlForAppStorageWithoutBackup()

            return url.path
        } catch {
            throw UtilitiesError.from(error)
        }
    }
}
