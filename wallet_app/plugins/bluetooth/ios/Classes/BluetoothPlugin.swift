import Flutter
import UIKit
import CoreBluetooth

public class BluetoothPlugin: NSObject, FlutterPlugin, CBCentralManagerDelegate {
  private var centralManager: CBCentralManager?
  private var currentState: CBManagerState = .unknown

  public static func register(with registrar: FlutterPluginRegistrar) {
    let channel = FlutterMethodChannel(name: "bluetooth", binaryMessenger: registrar.messenger())
    let instance = BluetoothPlugin()
    registrar.addMethodCallDelegate(instance, channel: channel)
  }

  public func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
    ensureCentralManager()
    switch call.method {
    case "isBluetoothEnabled":
      if(self.currentState == .unknown) {
        // Delay returning the result by 150ms to allow the state to initialize
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.15) { [weak self] in
          guard let self = self else { return }
          result(self.currentState == .poweredOn)
        }
      } else {
        result(self.currentState == .poweredOn)
      }
    default:
      result(FlutterMethodNotImplemented)
    }
  }
    
  private func ensureCentralManager() {
    guard centralManager == nil else { return }
    centralManager = CBCentralManager(
      delegate: self,
      queue: nil,
      options: [CBCentralManagerOptionShowPowerAlertKey: false]
    )
  }

  public func centralManagerDidUpdateState(_ central: CBCentralManager) {
    currentState = central.state
  }
}
