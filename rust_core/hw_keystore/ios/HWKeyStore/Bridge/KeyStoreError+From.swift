//
//  KeyStoreError+From.swift
//  HWKeyStore
//
//  Created by Wallet Developer on 27/02/2023.
//

import Foundation

extension KeyStoreError {
    static func from(_ error: SecureEnclaveKeyError) -> Self {
        switch error {
        case let .keychainError(message: message):
            return Self.KeyError(message: message)
        }
    }
}
