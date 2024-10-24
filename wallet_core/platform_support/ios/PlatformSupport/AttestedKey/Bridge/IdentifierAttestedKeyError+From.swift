//
//  IdentifierAttestedKeyError+From.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 24/10/2024.
//

import Foundation
import DeviceCheck

extension IdentifierAttestedKeyError {
    private static func retainIdentifier(for error: AppAttestError) -> Bool {
        guard #available(iOS 14, *),
              case let .attest(error) = error,
              let error = error as? DCError else {
            return false
        }

        return error.code == DCError.serverUnavailable
    }

    static func from(_ error: AppAttestError) -> Self {
        return .KeyError(reason: error.localizedDescription, retainIdentifier: self.retainIdentifier(for: error))
    }
}
