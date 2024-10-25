//
//  AttestedKeyError+From.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 24/10/2024.
//

import DeviceCheck
import Foundation

extension AttestedKeyError {
    static func from(_ error: DCError) -> Self {
        switch error.code {
        case DCError.featureUnsupported:
            return .AttestationNotSupported
        case DCError.serverUnavailable:
            return .ServerUnreachable(details: error.localizedDescription)
        default:
            return .Other(reason: error.localizedDescription)
        }
    }
}
