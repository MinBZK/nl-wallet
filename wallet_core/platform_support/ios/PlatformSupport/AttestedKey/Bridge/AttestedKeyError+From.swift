//
//  AttestedKeyError+From.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 24/10/2024.
//

import DeviceCheck
import Foundation

extension AttestedKeyError {
    static func from(_ error: AppAttestError) -> Self {
        switch error {
        case .unsupported:
            return .AttestationNotSupported
        case let .generate(innerError), let .attest(innerError), let .assert(innerError):
            guard let dcError = innerError as? DCError,
                  dcError.code == DCError.serverUnavailable else {
                return .Other(reason: error.localizedDescription)
            }

            return .ServerUnreachable(details: error.localizedDescription)
        }
    }
}
