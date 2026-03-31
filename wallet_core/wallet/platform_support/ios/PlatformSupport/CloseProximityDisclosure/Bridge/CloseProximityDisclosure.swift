//
//  CloseProximityDisclosure.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 06/03/2026.
//

import Foundation
@preconcurrency import Multipaz

final class CloseProximityDisclosure: @unchecked Sendable {
    let activeSessionLock = NSLock()
    let lifecycleLock = CloseProximityDisclosureLifecycleLock()
    let testingPeripheralServerModeUuid: Multipaz.UUID?
    // Background tasks only act on the session they were created for. Identity checks against the
    // current activeSession prevent stale work from a replaced session from emitting updates after
    // a newer handover has already started.
    var activeSession: CloseProximityDisclosureActiveSession?

    init(testingPeripheralServerModeUuid: Multipaz.UUID? = nil) {
        self.testingPeripheralServerModeUuid = testingPeripheralServerModeUuid
    }
}
