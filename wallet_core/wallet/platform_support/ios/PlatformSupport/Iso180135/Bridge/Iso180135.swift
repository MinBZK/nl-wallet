//
//  Iso180135.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 06/03/2026.
//

import Foundation

final class Iso180135 {}

extension Iso180135: Iso180135bridge {
    func startQrHandover(channel: Iso180135channel) async throws -> String {
        try await channel.sendUpdate(update: Iso180135update.connecting)

        try await channel.sendUpdate(update: Iso180135update.connected)

        try await channel.sendUpdate(
            update: Iso180135update.deviceRequest(
                sessionTranscript: [0x01, 0x02, 0x03], deviceRequest: [0x04, 0x05, 0x06]))

        try await channel.sendUpdate(update: Iso180135update.closed)

        return "some_qr_code"
    }

    func sendDeviceResponse(deviceResponse: [UInt8]) async throws {}

    func stopBleServer() async throws {}
}
