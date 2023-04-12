import UIKit
import PlatformSupport
import Flutter

@UIApplicationMain
@objc class AppDelegate: FlutterAppDelegate {
  var hardwareKeyStore: HWKeyStore?
  var utils: Utils?

  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    self.hardwareKeyStore = HWKeyStore.shared
    self.utils = Utils.shared

    let dummy = dummy_method_to_enforce_bundling()
    print(dummy)

    GeneratedPluginRegistrant.register(with: self)

    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
  }
}
