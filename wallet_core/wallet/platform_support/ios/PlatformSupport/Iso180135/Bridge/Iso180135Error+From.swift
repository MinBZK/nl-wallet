//
//  Iso180135+From.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 06/03/2026.
//

import Foundation

extension Iso180135error {
    static func from(_ error: Error) -> Self {
        return .PlatformError(reason: error.localizedDescription)
    }
}
