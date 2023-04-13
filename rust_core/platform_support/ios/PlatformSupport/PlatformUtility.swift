//
//  PlatformUtility.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 31/03/2023.
//

import Foundation

enum PlatformUtility {
    static let directoryName = "nobackup"

    static func urlForAppStorageWithoutBackup() throws -> URL {
        let fileManager = FileManager.default

        let appSupport = try fileManager.url(for: .applicationSupportDirectory,
                                             in: .userDomainMask,
                                             appropriateFor: nil,
                                             create: true)
        var url = appSupport.appendingPathComponent(self.directoryName, isDirectory: true)

        if !fileManager.fileExists(atPath: url.path) {
            try fileManager.createDirectory(at: url, withIntermediateDirectories: false)

            var values = URLResourceValues()
            values.isExcludedFromBackup = true

            do {
                try url.setResourceValues(values)
            } catch {
                try? fileManager.removeItem(at: url)

                throw error
            }
        }

        return url
    }
}
