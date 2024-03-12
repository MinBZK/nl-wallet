//
//  SecureEnclaveKeyError.swift
//  HWKeyStore
//
//  Created by Wallet Developer on 27/02/2023.
//

import Foundation

enum SecureEnclaveKeyError: Error {
    private static func format(message: String, with description: String?) -> String {
        guard let description else {
            return message
        }

        return "\(message): \(description)"
    }

    case fetch(errorMessage: String?)
    case create(keyChainError: Error?)
    case derivePublicKey(keyChainError: Error?)
    case sign(keyChainError: Error?)
    case encrypt(keyChainError: Error?)
    case decrypt(keyChainError: Error?)
    case delete(errorMessage: String?)

    var localizedDescription: String {
        switch self {
        case let .fetch(errorMessage: errorMessage):
            return Self.format(message: "Could not fetch private key", with: errorMessage)
        case let .create(keyChainError: keyChainError):
            return Self.format(message: "Could not create private key", with: keyChainError?.localizedDescription)
        case let .derivePublicKey(keyChainError: keyChainError):
            return Self.format(message: "Could not derive public key", with: keyChainError?.localizedDescription)
        case let .sign(keyChainError: keyChainError):
            return Self.format(message: "Could not sign with private key", with: keyChainError?.localizedDescription)
        case let .encrypt(keyChainError: keyChainError):
            return Self.format(message: "Could not encrypt", with: keyChainError?.localizedDescription)
        case let .decrypt(keyChainError: keyChainError):
            return Self.format(message: "Could not decrypt", with: keyChainError?.localizedDescription)
        case let .delete(errorMessage: errorMessage):
            return Self.format(message: "Could not delete private key", with: errorMessage)
        }
    }
}
