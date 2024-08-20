import UIKit
import PlatformSupport
import Flutter

@main
@objc class AppDelegate: FlutterAppDelegate {
  private var platformSupport: PlatformSupport?

  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    self.platformSupport = PlatformSupport.shared

    let dummy = dummy_method_to_enforce_bundling()
    print(dummy)

    GeneratedPluginRegistrant.register(with: self)

    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
  }
}
