//
//  AsymmetricKey.swift
//  HWKeyStore
//
//  Created by Wallet Developer on 24/02/2023.
//

import Foundation

final class AsymmetricKey {
    let key: SecureEnclaveKey

    init(key: SecureEnclaveKey) {
        self.key = key
    }
}

extension AsymmetricKey: AsymmetricKeyBridge {
    func publicKey() -> [UInt8] {
        return Array(self.key.publicKey)
    }

    func sign(payload: [UInt8]) -> [UInt8] {
        let signature = self.key.sign(payload: Data(payload))

        return Array(signature)
    }
}
