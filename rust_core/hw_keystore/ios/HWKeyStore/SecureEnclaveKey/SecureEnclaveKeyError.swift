//
//  SecureEnclaveKeyError.swift
//  HWKeyStore
//
//  Created by Wallet Developer on 27/02/2023.
//

import Foundation

enum SecureEnclaveKeyError: Error {
    case keychainError(message: String?)
}
