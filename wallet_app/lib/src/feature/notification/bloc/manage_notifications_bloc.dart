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

  late StreamSubscription _lifecycleSubscription;

  ManageNotificationsBloc(
    this._checkPermissionUseCase,
    this._requestPermissionUseCase,
    this._observeShowNotificationsSettingUsecase,
    this._setShowNotificationsSettingUsecase,
    this._lifecycleService,
  ) : super(const ManageNotificationsInitial()) {
    on<ManageNotificationsLoadTriggered>(_onRefresh);
    on<ManageNotificationsPushNotificationsToggled>(_onPushNotificationsToggled);
    _lifecycleSubscription = _lifecycleService
        .observe()
        .skip(1)
        .where((state) => state == .resumed)
        .listen((_) => add(const ManageNotificationsLoadTriggered()));
  }

  Future<void> _onRefresh(ManageNotificationsLoadTriggered event, Emitter<ManageNotificationsState> emit) async {
    final result = await _checkPermissionUseCase.invoke(.notification);
    final enabled = await _observeShowNotificationsSettingUsecase.invoke().first;
    // Also set and check local flag, clear flag on app reset?
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

    // Always (re)enable the internal setting
    await _setShowNotificationsSettingUsecase.invoke(enabled: true);

    // Check if permission is already granted
    final result = await _checkPermissionUseCase.invoke(.notification);
    if (result.isGranted) {
      // Permission already granted, simply update UI
      emit(const ManageNotificationsLoaded(pushEnabled: true));
      return;
    }

    // Check if it's permanently denied
    if (result.isPermanentlyDenied) {
      // No way to resolve this in-app, redirect to settings, refreshes in onResume
      unawaited(AppSettings.openAppSettings(type: .notification));
      return;
    }

    // Request in-app permission and emit result
    final requestResult = await _requestPermissionUseCase.invoke(.notification);
    emit(ManageNotificationsLoaded(pushEnabled: requestResult.isGranted));
  }

  @override
  Future<void> close() {
    _lifecycleSubscription.cancel();
    return super.close();
  }
}
