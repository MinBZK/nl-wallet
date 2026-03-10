//
//  CloseProximityDisclosureError+From.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 06/03/2026.
//

import Foundation

extension CloseProximityDisclosureError {
    static func from(_ error: Error) -> Self {
        return .PlatformError(reason: error.localizedDescription)
    }
}
