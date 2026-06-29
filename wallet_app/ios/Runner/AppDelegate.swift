import UIKit
import PlatformSupport
import Flutter
import Sentry
import flutter_local_notifications
import workmanager_apple

private let sentryMaxBreadcrumbs: UInt = 25
private let breadcrumbCategories = ["wallet.flow", "wallet.native"]
private let breadcrumbMessagePattern = try! NSRegularExpression(pattern: "^[a-z0-9_]+(\\.[a-z0-9_]+)*$")

@main
@objc class AppDelegate: FlutterAppDelegate, FlutterImplicitEngineDelegate {
  private var platformSupport: PlatformSupport?

  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    initializeSentry()

    self.platformSupport = PlatformSupport.shared

    let dummy = dummy_method_to_enforce_bundling()
    print(dummy)
    let dummy_frb = dummy_method_to_enforce_bundling_frb()
    print(dummy_frb)

    initializeLocalNotifications()
    initializeWorkmanager()

    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
  }

  fileprivate func initializeSentry() {
    let sentryConfig = dartDefines()
    guard let dsn = sentryConfig["SENTRY_DSN"], !dsn.isEmpty else { return }

    SentrySDK.start { options in
      options.dsn = dsn
      options.environment = sentryConfig["SENTRY_ENVIRONMENT"].flatMap { $0.isEmpty ? nil : $0 } ?? "unspecified"
      options.releaseName = sentryConfig["SENTRY_RELEASE"].flatMap { $0.isEmpty ? nil : $0 }
      #if DEBUG
      options.debug = true
      #else
      options.debug = false
      #endif
      options.sendDefaultPii = false
      options.enableCrashHandler = true
      options.enableWatchdogTerminationTracking = true
      options.enableAppHangTracking = true
      options.enableAppHangTrackingV2 = true
      options.maxBreadcrumbs = sentryMaxBreadcrumbs
      options.beforeBreadcrumb = { breadcrumb in
        guard self.isCuratedWalletBreadcrumb(breadcrumb) else { return nil }
        return self.sanitizeBreadcrumb(breadcrumb)
      }
      options.beforeSend = { event in
        event.user?.geo = nil
        event.user?.ipAddress = nil
        event.breadcrumbs = event.breadcrumbs?.compactMap { breadcrumb in
          guard self.isCuratedWalletBreadcrumb(breadcrumb) else { return nil }
          return self.sanitizeBreadcrumb(breadcrumb)
        }
        return event
      }
    }
  }

  fileprivate func dartDefines() -> [String: String] {
    guard let encodedDefines = Bundle.main.object(forInfoDictionaryKey: "DART_DEFINES") as? String else {
      return [:]
    }

    var result: [String: String] = [:]
    for encodedDefine in encodedDefines.split(separator: ",") {
      guard
        let data = Data(base64Encoded: String(encodedDefine)),
        let decoded = String(data: data, encoding: .utf8)
      else { continue }

      let parts = decoded.split(separator: "=", maxSplits: 1, omittingEmptySubsequences: false)
      guard parts.count == 2 else { continue }
      result[String(parts[0])] = String(parts[1])
    }
    return result
  }

  fileprivate func isCuratedWalletBreadcrumb(_ breadcrumb: Breadcrumb) -> Bool {
    guard breadcrumbCategories.contains(breadcrumb.category), let message = breadcrumb.message else { return false }

    let range = NSRange(location: 0, length: message.utf16.count)
    return breadcrumbMessagePattern.firstMatch(in: message, range: range)?.range == range
  }

  fileprivate func sanitizeBreadcrumb(_ breadcrumb: Breadcrumb) -> Breadcrumb {
    breadcrumb.data = nil
    breadcrumb.level = .info
    breadcrumb.type = "default"
    return breadcrumb
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

  fileprivate func initializeWorkmanager() {
    // Enable debug logging
    WorkmanagerDebug.setCurrent(LoggingDebugHandler())
    // Enable other plugins during background operations
    WorkmanagerPlugin.setPluginRegistrantCallback { registry in
      GeneratedPluginRegistrant.register(with: registry)
    }
    // Schedule hourly background-sync task
    WorkmanagerPlugin.registerPeriodicTask(
      withIdentifier: "nl.edi.wallet.background-sync",
      frequency: NSNumber(value: 3600) // 1 hour
    )
  }

  func didInitializeImplicitFlutterEngine(_ engineBridge: FlutterImplicitEngineBridge) {
    GeneratedPluginRegistrant.register(with: engineBridge.pluginRegistry)
  }
}
