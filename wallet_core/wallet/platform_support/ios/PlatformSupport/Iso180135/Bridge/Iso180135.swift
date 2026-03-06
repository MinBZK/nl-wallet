//
//  Iso180135Error+From.swift
//  PlatformSupport
//
//  Created by The Wallet Developers on 06/03/2026.
//

import Foundation

final class Iso180135 {}

extension Iso180135: Iso180135bridge {
    func startQrHandover(channel: Iso180135channel) async throws  -> String {
        try await channel.sendUpdate(update: Iso180135update.connecting)
        sleep(1)
        
        try await channel.sendUpdate(update: Iso180135update.connected)
        sleep(1)
        
        try await channel.sendUpdate(update: Iso180135update.deviceRequest(sessionTranscript: [], deviceRequest: []))
        sleep(1)
        
        try await channel.sendUpdate(update: Iso180135update.closed)
        sleep(1)

        return "some_qr_code"
    }

    func sendDeviceResponse(deviceResponse: [UInt8]) async throws {}
    
    func stopBleServer() async throws {}
}
