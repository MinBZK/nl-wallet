//
//  CloseProximityDisclosure.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 06/03/2026.
//

import Foundation

final class CloseProximityDisclosure {}

extension CloseProximityDisclosure: CloseProximityDisclosureBridge {
    func startQrHandover(channel: CloseProximityDisclosureChannel) async throws -> String {
        try await channel.sendUpdate(update: CloseProximityDisclosureUpdate.connecting)

        try await channel.sendUpdate(update: CloseProximityDisclosureUpdate.connected)

        try await channel.sendUpdate(
            update: CloseProximityDisclosureUpdate.sessionEstablished(
                sessionTranscript: [0x01, 0x02, 0x03], deviceRequest: [0x04, 0x05, 0x06]))

        try await channel.sendUpdate(update: CloseProximityDisclosureUpdate.closed)

        return "some_qr_code"
    }

    func sendDeviceResponse(deviceResponse: [UInt8]) async throws {}

    func stopBleServer() async throws {}
}
