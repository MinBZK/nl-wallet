//
//  CloseProximityDisclosure.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 06/03/2026.
//

import Foundation

actor CloseProximityDisclosure {
    // Serializes lifecycle transactions such as startQrHandover() and stopBleServer() across
    // suspension points so start/stop cannot interleave and leave BLE/session state half updated.
    // Example race prevented: startQrHandover() installs a new session while an overlapping
    // stopBleServer() is still canceling and closing the previous BLE transport, causing the new
    // handover to inherit teardown work from the old one.
    let lifecycleLock = CloseProximityDisclosureLifecycleLock()
    let testingPeripheralServerModeUuid: UUID?
    // The actor owns the current session, including the mutable runtime state derived while the
    // BLE exchange is active. Background work hops back into the actor to touch this state.
    var activeSession: CloseProximityDisclosureActiveSession?

    init(testingPeripheralServerModeUuid: UUID? = nil) {
        self.testingPeripheralServerModeUuid = testingPeripheralServerModeUuid
    }
}
