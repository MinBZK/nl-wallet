import 'dart:async';

import 'package:app_settings/app_settings.dart';
import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/service/app_lifecycle_service.dart';
import '../../../domain/usecase/notification/observe_push_notifications_setting_usecase.dart';
import '../../../domain/usecase/notification/set_push_notifications_setting_usecase.dart';
import '../../../domain/usecase/permission/check_permission_usecase.dart';
import '../../../domain/usecase/permission/request_permission_usecase.dart';
import '../../../util/cast_util.dart';

part 'manage_notifications_event.dart';
part 'manage_notifications_state.dart';

class ManageNotificationsBloc extends Bloc<ManageNotificationsEvent, ManageNotificationsState> {
  final CheckPermissionUseCase _checkPermissionUseCase;
  final RequestPermissionUseCase _requestPermissionUseCase;
  final ObservePushNotificationsSettingUseCase _observeShowNotificationsSettingUsecase;
  final SetPushNotificationsSettingUseCase _setShowNotificationsSettingUsecase;
  final AppLifecycleService _lifecycleService;

  /// Subscription to the app lifecycle stream. Used to cancel the stream when the bloc is closed.
  late StreamSubscription _lifecycleSubscription;

  /// Flag that indicates we redirected the user to the App's settings. Used in the onResume callback.
  bool _didRedirectToSettings = false;

  ManageNotificationsBloc(
    this._checkPermissionUseCase,
    this._requestPermissionUseCase,
    this._observeShowNotificationsSettingUsecase,
    this._setShowNotificationsSettingUsecase,
    this._lifecycleService,
  ) : super(const ManageNotificationsInitial()) {
    on<ManageNotificationsLoadTriggered>(_onRefresh);
    on<ManageNotificationsPushNotificationsToggled>(_onPushNotificationsToggled);
    _lifecycleSubscription = _lifecycleService.observe().skip(1).where((state) => state == .resumed).listen((_) {
      add(ManageNotificationsLoadTriggered(isRefreshAfterSettingsRedirect: _didRedirectToSettings));
      _didRedirectToSettings = false;
    });
  }

  Future<void> _onRefresh(ManageNotificationsLoadTriggered event, Emitter<ManageNotificationsState> emit) async {
    final result = await _checkPermissionUseCase.invoke(.notification);

    /// Check for the case where user was redirected to the settings to grant the permission
    if (event.isRefreshAfterSettingsRedirect && result.isGranted) {
      await _setShowNotificationsSettingUsecase.invoke(enabled: true);
    }

    /// Emit the current state
    final enabled = await _observeShowNotificationsSettingUsecase.invoke().first;
    emit(ManageNotificationsLoaded(pushEnabled: enabled && result.isGranted));
  }

  Future<void> _onPushNotificationsToggled(
    ManageNotificationsPushNotificationsToggled event,
    Emitter<ManageNotificationsState> emit,
  ) async {
    final currentlyEnabled = tryCast<ManageNotificationsLoaded>(state)?.pushEnabled ?? false;
    if (currentlyEnabled) {
      // Simply disable the internal setting and update the UI
      await _setShowNotificationsSettingUsecase.invoke(enabled: false);
      emit(const ManageNotificationsLoaded(pushEnabled: false));
      return;
    }

    // Check if permission is already granted
    final result = await _checkPermissionUseCase.invoke(.notification);
    if (result.isGranted) {
      // Permission already granted, update setting & ui
      await _setShowNotificationsSettingUsecase.invoke(enabled: true);
      emit(const ManageNotificationsLoaded(pushEnabled: true));
      return;
    }

    // Check if it's permanently denied
    if (result.isPermanentlyDenied) {
      // No way to resolve this in-app, redirect to settings, refresh in onResume.
      unawaited(AppSettings.openAppSettings(type: .notification));
      _didRedirectToSettings = true;
      return;
    }

    // Request in-app permission and handle the result
    final requestResult = await _requestPermissionUseCase.invoke(.notification);
    await _setShowNotificationsSettingUsecase.invoke(enabled: requestResult.isGranted);
    emit(ManageNotificationsLoaded(pushEnabled: requestResult.isGranted));
  }

  @override
  Future<void> close() {
    _lifecycleSubscription.cancel();
    return super.close();
  }
}
