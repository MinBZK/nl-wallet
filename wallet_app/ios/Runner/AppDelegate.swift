import UIKit
import PlatformSupport
import Flutter
import flutter_local_notifications

@main
@objc class AppDelegate: FlutterAppDelegate, FlutterImplicitEngineDelegate {
  private var platformSupport: PlatformSupport?

  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    self.platformSupport = PlatformSupport.shared

    let dummy = dummy_method_to_enforce_bundling()
    print(dummy)
    let dummy_frb = dummy_method_to_enforce_bundling_frb()
    print(dummy_frb)

    initializeLocalNotifications()

    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
  }
    
  fileprivate func initializeLocalNotifications() {
    if #available(iOS 10.0, *) {
      UNUserNotificationCenter.current().delegate = self as? UNUserNotificationCenterDelegate
    }
    // Make sure notifications don't persist through re-installs
    let notificationInitializedKey = "local_notifications_initialized"
    if (!UserDefaults.standard.bool(forKey: notificationInitializedKey)) {
      UIApplication.shared.cancelAllLocalNotifications()
      UserDefaults.standard.set(true, forKey: notificationInitializedKey)
    }
  }
  
  func didInitializeImplicitFlutterEngine(_ engineBridge: FlutterImplicitEngineBridge) {
    GeneratedPluginRegistrant.register(with: engineBridge.pluginRegistry)
  }
}
