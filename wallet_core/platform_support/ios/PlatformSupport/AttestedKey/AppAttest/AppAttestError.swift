//
//  AppAttestError.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 24/10/2024.
//

import Foundation

enum AppAttestError: Error {
    private static func format(message: String, with error: Error?) -> String {
        guard let error else {
            return message
        }

        return "\(message): \(error.localizedDescription)"
    }

    case unsupported
    case generate(error: Error?)
    case attest(error: Error?)
    case assert(error: Error?)

    var localizedDescription: String {
        switch self {
        case .unsupported:
            return "AppAttest is not supported on this device"
        case let .generate(error):
            return Self.format(message: "Could not generate key", with: error)
        case let .attest(error):
            return Self.format(message: "Could not attest key", with: error)
        case let .assert(error):
            return Self.format(message: "Could not generate assertion", with: error)
        }
    }
}
