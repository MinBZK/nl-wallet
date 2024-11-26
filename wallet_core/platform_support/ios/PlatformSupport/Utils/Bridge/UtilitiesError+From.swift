//
//  UtilitiesError+From.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 31/03/2023.
//

import Foundation

extension UtilitiesError {
    static func from(_ error: Error) -> Self {
        return .PlatformError(reason: error.localizedDescription)
    }
}
