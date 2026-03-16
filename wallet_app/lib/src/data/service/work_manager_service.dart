import 'dart:async';
import 'dart:io';

import 'package:fimber/fimber.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wallet_core/core.dart' as core;
import 'package:workmanager/workmanager.dart';

import '../../../l10n/generated/app_localizations.dart';
import '../../domain/model/notification/notification_channel.dart';
import '../../domain/model/notification/os_notification.dart';
import '../../domain/usecase/notification/impl/set_direct_os_notification_callback_usecase_impl.dart';
import '../../util/extension/locale_extension.dart';
import '../../util/mapper/card/attribute/card_attribute_mapper.dart';
import '../../util/mapper/card/attribute/card_attribute_value_mapper.dart';
import '../../util/mapper/card/attribute/claim_display_metadata_mapper.dart';
import '../../util/mapper/card/attribute/localized_labels_mapper.dart';
import '../../util/mapper/card/card_mapper.dart';
import '../../util/mapper/card/metadata_mapper.dart';
import '../../util/mapper/card/status/card_status_mapper.dart';
import '../../util/mapper/image/image_mapper.dart';
import '../../util/mapper/notification/app_notification_mapper.dart';
import '../../util/mapper/notification/notification_display_target_mapper.dart';
import '../../util/mapper/notification/notification_type_mapper.dart';
import '../../util/mapper/organization/organization_mapper.dart';
import '../../wallet_constants.dart';
import '../../wallet_core/error/core_error_mapper.dart';
import '../../wallet_core/typed/typed_wallet_core.dart';
import '../repository/language/impl/language_repository_impl.dart';
import '../repository/notification/impl/notification_repository_impl.dart';
import '../store/active_locale_provider.dart';
import '../store/impl/active_localization_delegate.dart';
import '../store/impl/language_store_impl.dart';
import '../store/impl/notification_settings_store_impl.dart';

const kDebugTag = 'WorkManager';

const kBackgroundSyncTask = 'nl.edi.wallet.background-sync';
const kBackgroundSyncTaskIds = [kBackgroundSyncTask, Workmanager.iOSBackgroundTask];

/// A service that wraps the [Workmanager] plugin to implement background sync.
///
/// The actual sync happens in native Rust code, but this class ensures
/// dependencies are provided and the sync method is called when the background
/// isolate is spawned.
class WorkManagerService {
  final Workmanager _workmanager;

  /// Initializes [Workmanager] and schedules periodic background sync tasks.
  WorkManagerService(this._workmanager) {
    _workmanager.initialize(callbackDispatcher).then((_) async {
      await _workmanager.registerPeriodicTask(
        kBackgroundSyncTask,
        kBackgroundSyncTask,
        existingWorkPolicy: .update,
        initialDelay: const Duration(minutes: 5),
        constraints: Constraints(networkType: .connected),
        frequency: const Duration(hours: 1),
        tag: 'background-sync',
      );

      if (Platform.isIOS) {
        final scheduledTasks = await _workmanager.printScheduledTasks();
        Fimber.d('[$kDebugTag] Finished scheduling\n$scheduledTasks');
      }
    });
  }
}

/// Entry point for the background isolate, required by the [Workmanager] plugin.
@pragma('vm:entry-point')
void callbackDispatcher() {
  if (kDebugMode) Fimber.plantTree(DebugTree());
  Workmanager().executeTask(executeTask);
}

@visibleForTesting
Future<bool> executeTask(String task, Map<String, dynamic>? inputData) async {
  final startTime = DateTime.now();
  Fimber.d('[$kDebugTag] 🚀 Task started: $task. Input data: $inputData');

  if (!kBackgroundSyncTaskIds.contains(task)) {
    Fimber.e('[$kDebugTag] Unknown task: $task, aborting.');
    return true;
  }

  try {
    await performRevocationCheckTask();

    final duration = DateTime.now().difference(startTime);
    Fimber.d('[$kDebugTag] ✅ Task completed in ${duration.inSeconds}s');

    return true; // Report completed successfully
  } catch (error, stackTrace) {
    final duration = DateTime.now().difference(startTime);
    Fimber.e('[$kDebugTag] ❌ Task failed after ${duration.inSeconds}s', ex: error, stacktrace: stackTrace);
    return false; // Schedule retry
  }
}

/// Sets up dependencies and triggers the native background synchronization.
///
/// By default it also initializes the Rust library, however [initCore] can be set to false
/// to allow testing during normal app runtime (i.e. to avoid initializing it twice).
Future<void> performRevocationCheckTask({bool initCore = true}) async {
  /// Initialize notification plugin
  final plugin = FlutterLocalNotificationsPlugin();
  await plugin.initialize(const InitializationSettings(android: kAndroidInitSettings, iOS: kDarwinInitSettings));

  /// Initialize wallet_core
  if (initCore) {
    final lib = Platform.isIOS || Platform.isMacOS ? ExternalLibrary.process(iKnowHowToUseIt: true) : null;
    await core.WalletCore.init(externalLibrary: lib);
  }

  /// Initialize dependencies
  final imageMapper = ImageMapper();
  final notificationTypeMapper = NotificationTypeMapper(
    CardMapper(
      CardAttributeMapper(CardAttributeValueMapper(), ClaimDisplayMetadataMapper()),
      OrganizationMapper(LocalizedLabelsMapper(), imageMapper),
      DisplayMetadataMapper(imageMapper),
      CardStatusMapper(),
    ),
  );
  final notificationRepository = NotificationRepositoryImpl(
    TypedWalletCore(CoreErrorMapper()),
    AppNotificationMapper(notificationTypeMapper, NotificationDisplayTargetMapper()),
    notificationTypeMapper,
    NotificationSettingsStoreImpl(SharedPreferences.getInstance),
  );
  final activeLocaleProvider = await _localeProvider();
  final directNotificationsUsecase = SetDirectOsNotificationCallbackUsecaseImpl(
    notificationRepository,
    activeLocaleProvider,
  );

  /// Setup notification callback
  directNotificationsUsecase.invoke(
    (notification) => _onDirectNotification(plugin, activeLocaleProvider, notification),
  );

  /// Perform actual processing
  Fimber.d('[$kDebugTag] 📝 Dependencies initialized, delegating sync to wallet_core');
  await core.performBackgroundSync();
}

/// Provides the [ActiveLocaleProvider] for the background isolate.
Future<ActiveLocaleProvider> _localeProvider() async {
  final activeLocalizationDelegate = ActiveLocalizationDelegate();
  final defaultLocale = PlatformDispatcher.instance.locale;
  // Fetch override locale (if set)
  final languageStoreImpl = LanguageStoreImpl(SharedPreferences.getInstance);
  final languageRepository = LanguageRepositoryImpl(languageStoreImpl, AppLocalizations.supportedLocales);
  // Update activeLocalizationDelegate with relevant locale
  await activeLocalizationDelegate.load(await languageRepository.preferredLocale.first ?? defaultLocale);
  return activeLocalizationDelegate;
}

/// Callback invoked when a notification is triggered directly from native code.
void _onDirectNotification(
  FlutterLocalNotificationsPlugin plugin,
  ActiveLocaleProvider provider,
  OsNotification notification,
) {
  final details = NotificationDetails(
    android: resolveAndroidDetails(provider, notification.channel),
    iOS: const DarwinNotificationDetails(presentAlert: true),
  );
  plugin.show(notification.id, notification.title, notification.body, details, payload: notification.payload);
}

/// Resolves the [AndroidNotificationDetails] for a given [channel].
@visibleForTesting
AndroidNotificationDetails? resolveAndroidDetails(
  ActiveLocaleProvider activeLocaleProvider,
  NotificationChannel channel,
) {
  return switch (channel) {
    NotificationChannel.cardUpdates => AndroidNotificationDetails(
      channel.name,
      activeLocaleProvider.activeLocale.l10n.cardNotificationsChannelName,
      channelDescription: activeLocaleProvider.activeLocale.l10n.cardNotificationsChannelDescription,
      autoCancel: true,
    ),
  };
}
