//
//  PlatformUtility.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 31/03/2023.
//

import Foundation

enum PlatformUtility {
    private static let directoryName = "nobackup"

    private static let queue = DispatchQueue(label: String(describing: Self.self), qos: .userInitiated)
    private static var storageUrl: URL?

    static func urlForAppStorageWithoutBackup() throws -> URL {
        // cache URL in self.storageUrl
        guard let url = self.storageUrl else {
            // if not present, create the URL and store in cache
            // creation is made thread safe by a serial queue
            let url = try self.queue.sync(execute: self.createStorageUrl)
            self.storageUrl = url

            return url
        }

        return url
    }

    private static func createStorageUrl() throws -> URL {
        let fileManager = FileManager.default

        // get the "Application Support/nobackup" dir
        let appSupport = try fileManager.url(for: .applicationSupportDirectory,
                                             in: .userDomainMask,
                                             appropriateFor: nil,
                                             create: true)
        var url = appSupport.appendingPathComponent(self.directoryName, isDirectory: true)

        // if it does not exist...
        if !fileManager.fileExists(atPath: url.path) {
            // ...create it and...
            try fileManager.createDirectory(at: url, withIntermediateDirectories: false)

            // ...exclude its contents from backups
            var values = URLResourceValues()
            values.isExcludedFromBackup = true

            do {
                try url.setResourceValues(values)
            } catch {
                // delete the folder again if setting this flag fails
                try? fileManager.removeItem(at: url)

                throw error
            }
        }

        return url
    }
}
