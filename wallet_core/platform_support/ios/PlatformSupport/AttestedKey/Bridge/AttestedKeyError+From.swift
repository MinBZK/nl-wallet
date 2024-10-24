//
//  AttestedKeyError+From.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 24/10/2024.
//

import Foundation

extension AttestedKeyError {
    static func from(_ error: AppAttestError) -> Self {
        return .KeyError(reason: error.localizedDescription)
    }
}
