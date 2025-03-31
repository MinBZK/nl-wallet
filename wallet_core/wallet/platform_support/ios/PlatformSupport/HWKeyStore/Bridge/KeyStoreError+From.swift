//
//  KeyStoreError+From.swift
//  HWKeyStore
//
//  Created by Wallet Developer on 27/02/2023.
//

import Foundation

extension KeyStoreError {
    static func from(_ error: SecureEnclaveKeyError) -> Self {
        return .KeyError(reason: error.localizedDescription)
    }
}
