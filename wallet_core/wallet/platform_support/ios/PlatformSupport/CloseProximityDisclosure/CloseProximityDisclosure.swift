//
//  CloseProximityDisclosure.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 06/03/2026.
//

import Foundation
@preconcurrency import Multipaz

actor CloseProximityDisclosure {
    // Serializes lifecycle transactions such as startQrHandover() and stopBleServer() across
    // suspension points so start/stop cannot interleave and leave BLE/session state half updated.
    // Example race prevented: startQrHandover() installs a new session while an overlapping
    // stopBleServer() is still canceling and closing the previous BLE transport, causing the new
    // handover to inherit teardown work from the old one.
    let lifecycleLock = CloseProximityDisclosureLifecycleLock()
    let testingPeripheralServerModeUuid: Multipaz.UUID?
    // The actor owns the current session and all mutable runtime state associated with it:
    // background task handles plus the session-encryption/transcript derived from the reader's
    // first message. Background work hops back into the actor to touch this state.
    var activeSessionState: CloseProximityDisclosureActiveSessionState?

    init(testingPeripheralServerModeUuid: Multipaz.UUID? = nil) {
        self.testingPeripheralServerModeUuid = testingPeripheralServerModeUuid
    }
}
