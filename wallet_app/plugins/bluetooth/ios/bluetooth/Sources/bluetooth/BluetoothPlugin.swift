import Flutter
import UIKit
import CoreBluetooth

public class BluetoothPlugin: NSObject, FlutterPlugin, FlutterStreamHandler, CBCentralManagerDelegate {
  private var centralManager: CBCentralManager?
  private var currentState: CBManagerState = .unknown
  private var stateSink: FlutterEventSink?

  public static func register(with registrar: FlutterPluginRegistrar) {
    let channel = FlutterMethodChannel(name: "bluetooth", binaryMessenger: registrar.messenger())
    let stateChannel = FlutterEventChannel(name: "bluetooth/state", binaryMessenger: registrar.messenger())
    let instance = BluetoothPlugin()
    registrar.addMethodCallDelegate(instance, channel: channel)
    stateChannel.setStreamHandler(instance)
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

  public func onListen(withArguments arguments: Any?, eventSink events: @escaping FlutterEventSink) -> FlutterError? {
    stateSink = events
    ensureCentralManager()
    emitState()
    return nil
  }

  public func onCancel(withArguments arguments: Any?) -> FlutterError? {
    stateSink = nil
    return nil
  }

  private func ensureCentralManager() {
    guard centralManager == nil else { return }
    centralManager = CBCentralManager(
      delegate: self,
      queue: nil,
      options: [CBCentralManagerOptionShowPowerAlertKey: false]
    )
  }

  // Right after Bluetooth is switched on, the state is still poweredOff or resetting; this
  // callback is the only reliable moment to learn that it became poweredOn.
  public func centralManagerDidUpdateState(_ central: CBCentralManager) {
    currentState = central.state
    emitState()
  }

  private func emitState() {
    guard currentState != .unknown else { return }
    stateSink?(currentState == .poweredOn)
  }
}
