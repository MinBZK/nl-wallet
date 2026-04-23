import Flutter
import UIKit
import CoreBluetooth

public class BluetoothPlugin: NSObject, FlutterPlugin, CBCentralManagerDelegate {
  private var centralManager: CBCentralManager!
  private var currentState: CBManagerState = .unknown

  override init() {
    super.init()
    centralManager = CBCentralManager(delegate: self, queue: nil)
  }

  public static func register(with registrar: FlutterPluginRegistrar) {
    let channel = FlutterMethodChannel(name: "bluetooth", binaryMessenger: registrar.messenger())
    let instance = BluetoothPlugin()
    registrar.addMethodCallDelegate(instance, channel: channel)
  }

  public func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
    switch call.method {
    case "isBluetoothEnabled":
      result(currentState == .poweredOn)
    default:
      result(FlutterMethodNotImplemented)
    }
  }

  public func centralManagerDidUpdateState(_ central: CBCentralManager) {
    currentState = central.state
  }
}
