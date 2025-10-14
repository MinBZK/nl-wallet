import 'dart:async';

import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:rxdart/subjects.dart';

import '../../domain/model/update/update_notification.dart';
import '../../domain/model/wallet_status.dart';
import '../../domain/usecase/navigation/check_navigation_prerequisites_usecase.dart';
import '../../domain/usecase/navigation/perform_pre_navigation_actions_usecase.dart';
import '../../domain/usecase/transfer/cancel_wallet_transfer_usecase.dart';
import '../../domain/usecase/wallet/get_wallet_status_usecase.dart';
import '../../feature/common/dialog/idle_warning_dialog.dart';
import '../../feature/common/dialog/locked_out_dialog.dart';
import '../../feature/common/dialog/reset_wallet_dialog.dart';
import '../../feature/common/dialog/scan_with_wallet_dialog.dart';
import '../../feature/common/dialog/update_notification_dialog.dart';
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
  final GetWalletStatusUseCase _getWalletStatusUseCase;
  final CancelWalletTransferUseCase _cancelWalletTransferUseCase;

  NavigationService(
    this._navigatorKey,
    this._checkNavigationPrerequisitesUseCase,
    this._performPreNavigationActionsUseCase,
    this._getWalletStatusUseCase,
    this._cancelWalletTransferUseCase,
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
    if (request.removeUntil == null) {
      await _navigatorKey.currentState?.pushNamed(request.destination, arguments: request.argument);
    } else {
      await _navigatorKey.currentState?.pushNamedAndRemoveUntil(
        request.destination,
        ModalRoute.withName(request.removeUntil!),
        arguments: request.argument,
      );
    }
  }

  /// Initialization related hook, called whenever the dashboard is shown to the user. E.g. right after unlocking the
  /// app, but also when the user arrives back on the dashboard after a disclosure/issuance/etc. flow.
  Future<void> notifyDashboardShown() async {
    final WalletStatus status = await _getWalletStatusUseCase.invoke();
    switch (status) {
      case WalletStatusTransferring():
        if (status.role == TransferRole.target) _queuedRequest = NavigationRequest.walletTransferTarget(isRetry: true);
        await _cancelWalletTransferUseCase.invoke();
      case WalletStatusReady():
        break;
    }
    await processQueue();
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
        WarnUpdateNotification() => UpdateNotificationDialog.show(
          context,
          timeUntilBlocked: notification.timeUntilBlocked,
        ),
      };
    } finally {
      // Register that the dialog is no longer visible
      _updateNotificationDialogVisible.add(false);
    }
  }

  Stream<bool> observeUpdateNotificationDialogVisible() => _updateNotificationDialogVisible.stream.distinct();
}

enum WalletDialogType { idleWarning, resetWallet, scanWithWallet, lockedOut }
