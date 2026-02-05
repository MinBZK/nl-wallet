import '../../model/notification/os_notification.dart';
import '../wallet_usecase.dart';

/// Observes notifications from the wallet's core that are intended to be displayed
/// as OS notifications.
///
/// This use case transforms raw `AppNotification` data into a list of `OsNotification`
/// objects, which are ready to be scheduled with the operating system's notification service.
///
/// The stream can be configured to respect the user's in-app notification settings.
abstract class ObserveOsNotificationsUseCase extends WalletUseCase {
  /// Returns a stream of [OsNotification]s.
  ///
  /// - [respectUserSetting]: When `true` (the default), the stream will only
  ///   emit notifications if the user has enabled push notifications in the app's
  ///   settings. If disabled, it will emit an empty list. When `false`, it will
  ///   emit notifications regardless of the user's preference. This is useful for
  ///   scenarios where you need to know about scheduled notifications without
  ///   being affected by the user's preference, such as a debug screen.
  Stream<List<OsNotification>> invoke({bool respectUserSetting = true});
}
