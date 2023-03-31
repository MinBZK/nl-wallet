//
//  Utils.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 31/03/2023.
//

import Foundation

public final class Utils {
    public static let shared = Utils()

    private let utilities: Utilities

    private init() {
        self.utilities = Utilities()

        initUtilities(bridge: self.utilities)
    }
}
