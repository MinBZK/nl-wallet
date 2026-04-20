//
//  CloseProximityDisclosure.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 06/03/2026.
//

import Foundation
@preconcurrency import Multipaz

final class CloseProximityDisclosure: @unchecked Sendable {
    // Guards the single "current session" slot on CloseProximityDisclosure. There is only one
    // active session at a time, but older session objects and their background tasks can still be
    // alive briefly after replacement or shutdown and must compare against this slot safely.
    // Example race prevented: session A's read task wakes up just after session B becomes active
    // and incorrectly clears or reports against B's slot while checking whether A is still current.
    let activeSessionLock = NSLock()
    // Serializes lifecycle transactions such as startQrHandover() and stopBleServer() across
    // suspension points so start/stop cannot interleave and leave BLE/session state half updated.
    // Example race prevented: startQrHandover() installs a new session while an overlapping
    // stopBleServer() is still canceling and closing the previous BLE transport, causing the new
    // handover to inherit teardown work from the old one.
    let lifecycleLock = CloseProximityDisclosureLifecycleLock()
    let testingPeripheralServerModeUuid: Multipaz.UUID?
    // The current session slot. Background tasks only act on the session they were created for,
    // and identity checks against this reference prevent stale work from a replaced session from
    // emitting updates after a newer handover has already started.
    var activeSession: CloseProximityDisclosureActiveSession?

    init(testingPeripheralServerModeUuid: Multipaz.UUID? = nil) {
        self.testingPeripheralServerModeUuid = testingPeripheralServerModeUuid
    }
}
