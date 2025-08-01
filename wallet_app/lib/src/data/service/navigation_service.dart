import 'dart:async';

import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:rxdart/subjects.dart';

import '../../domain/model/update/update_notification.dart';
import '../../domain/usecase/navigation/check_navigation_prerequisites_usecase.dart';
import '../../domain/usecase/navigation/perform_pre_navigation_actions_usecase.dart';
import '../../feature/common/dialog/idle_warning_dialog.dart';
import '../../feature/common/dialog/locked_out_dialog.dart';
import '../../feature/common/dialog/reset_wallet_dialog.dart';
import '../../feature/common/dialog/scan_with_wallet_dialog.dart';
import '../../feature/common/dialog/update_notification_dialog.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/helper/dialog_helper.dart';

class NavigationService {
  /// Key that holds [NavigatorState], used to perform navigation from a non-Widget.
  final GlobalKey<NavigatorState> _navigatorKey;

  /// A queued [NavigationRequest], when navigation can't be performed (e.g. app
  /// not in a state where it can be handled) maximum 1 [NavigationRequest] is queued
  /// here to be handled when [processQueue] is called.
  NavigationRequest? _queuedRequest;

  /// Stream that emits whether the update notification dialog is visible.
  final BehaviorSubject<bool> _updateNotificationDialogVisible = BehaviorSubject.seeded(false);

  final CheckNavigationPrerequisitesUseCase _checkNavigationPrerequisitesUseCase;
  final PerformPreNavigationActionsUseCase _performPreNavigationActionsUseCase;

  NavigationService(
    this._navigatorKey,
    this._checkNavigationPrerequisitesUseCase,
    this._performPreNavigationActionsUseCase,
  );

  /// Process the provided [NavigationRequest], or queue it if the app is in a state where it can't be handled.
  /// Overrides any previously set [NavigationRequest] if this request has to be queued as well.
  Future<void> handleNavigationRequest(NavigationRequest request, {bool queueIfNotReady = false}) async {
    final readyToNavigate = await _checkNavigationPrerequisitesUseCase.invoke(request.navigatePrerequisites);
    if (readyToNavigate) {
      await _navigate(request);
    } else {
      if (queueIfNotReady) _queuedRequest = request;
      Fimber.d('Not yet ready to handle $request. Request queued: $queueIfNotReady');
    }
  }

  Future<void> _navigate(NavigationRequest request) async {
    assert(
      await _checkNavigationPrerequisitesUseCase.invoke(request.navigatePrerequisites),
      'NavigationPreRequisites should have been validated before calling _navigate!',
    );
    await _performPreNavigationActionsUseCase.invoke(request.preNavigationActions);
    switch (request) {
      case PidIssuanceNavigationRequest():
        await _navigatorKey.currentState?.pushNamedAndRemoveUntil(
          request.destination,
          ModalRoute.withName(WalletRoutes.splashRoute),
          arguments: request.argument,
        );
      case PidRenewalNavigationRequest():
        await _navigatorKey.currentState?.pushNamedAndRemoveUntil(
          request.destination,
          ModalRoute.withName(WalletRoutes.cardDetailRoute),
          arguments: request.argument,
        );
      case DisclosureNavigationRequest():
      case IssuanceNavigationRequest():
      case SignNavigationRequest():
        await _navigatorKey.currentState?.pushNamedAndRemoveUntil(
          request.destination,
          ModalRoute.withName(WalletRoutes.dashboardRoute),
          arguments: request.argument,
        );
      case GenericNavigationRequest():
        await _navigatorKey.currentState?.pushNamed(
          request.destination,
          arguments: request.argument,
        );
    }
  }

  /// Consume and process the outstanding [NavigationRequest].
  Future<void> processQueue() async {
    if (_queuedRequest == null) return;
    assert(
      await _checkNavigationPrerequisitesUseCase.invoke(NavigationPrerequisite.values),
      'processQueue() should only be called when all prerequisites have been met.',
    );
    final queuedRequest = _queuedRequest;
    _queuedRequest = null;
    if (queuedRequest != null) await handleNavigationRequest(queuedRequest);
  }

  /// Show the dialog specified by [type]. Useful when caller does not have a valid context.
  Future<void> showDialog(WalletDialogType type, {bool dismissOpenDialogs = false}) async {
    final context = _navigatorKey.currentState?.context;
    if (context == null || !context.mounted) return;
    if (dismissOpenDialogs) await DialogHelper.dismissOpenDialogs(context);
    if (!context.mounted) return; // Fixes lint warning: context across async gaps
    return switch (type) {
      WalletDialogType.idleWarning => IdleWarningDialog.show(context),
      WalletDialogType.resetWallet => ResetWalletDialog.show(context),
      WalletDialogType.scanWithWallet => ScanWithWalletDialog.show(context),
      WalletDialogType.lockedOut => LockedOutDialog.show(context),
    };
  }

  Future<void> processUpdateNotification(UpdateNotification notification) async {
    final context = _navigatorKey.currentState?.context;
    if (context == null || !context.mounted) return;

    // Register that the dialog is visible
    _updateNotificationDialogVisible.add(true);

    // Show the dialog
    try {
      await switch (notification) {
        RecommendUpdateNotification() => UpdateNotificationDialog.show(context),
        WarnUpdateNotification() =>
          UpdateNotificationDialog.show(context, timeUntilBlocked: notification.timeUntilBlocked),
      };
    } finally {
      // Register that the dialog is no longer visible
      _updateNotificationDialogVisible.add(false);
    }
  }

  Stream<bool> observeUpdateNotificationDialogVisible() => _updateNotificationDialogVisible.stream.distinct();
}

enum WalletDialogType { idleWarning, resetWallet, scanWithWallet, lockedOut }
