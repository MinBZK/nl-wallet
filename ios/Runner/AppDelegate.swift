import UIKit
import HWKeyStore
import Flutter

@UIApplicationMain
@objc class AppDelegate: FlutterAppDelegate {
  var hardwareKeyStore: HWKeyStore?

  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    self.hardwareKeyStore = HWKeyStore.shared

    let dummy = dummy_method_to_enforce_bundling()
    print(dummy)

    GeneratedPluginRegistrant.register(with: self)

    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
  }
}
