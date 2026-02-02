import 'dart:async';

import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';

import '../../domain/app_event/app_event_listener.dart';
import '../../domain/model/update/update_notification.dart';
import '../../domain/model/wallet_state.dart';
import '../../domain/usecase/navigation/check_navigation_prerequisites_usecase.dart';
import '../../domain/usecase/navigation/perform_pre_navigation_actions_usecase.dart';
import '../../domain/usecase/wallet/get_wallet_state_usecase.dart';
import '../../feature/common/dialog/generic_dialog.dart';
import '../../feature/common/dialog/idle_warning_dialog.dart';
import '../../feature/common/dialog/locked_out_dialog.dart';
import '../../feature/common/dialog/move_stopped_dialog.dart';
import '../../feature/common/dialog/reset_wallet_dialog.dart';
import '../../feature/common/dialog/scan_with_wallet_dialog.dart';
import '../../feature/common/dialog/update_notification_dialog.dart';
import '../../feature/notification/sheet/request_notification_permission_sheet.dart';
import '../../util/helper/dialog_helper.dart';

class NavigationService extends AppEventListener {
  /// Key that holds [NavigatorState], used to perform navigation from a non-Widget.
  final GlobalKey<NavigatorState> _navigatorKey;

  /// A queued [NavigationRequest], when navigation can't be performed (e.g. app
  /// not in a state where it can be handled) maximum 1 [NavigationRequest] is queued
  /// here to be handled when [processQueue] is called.
  NavigationRequest? _queuedRequest;

  final CheckNavigationPrerequisitesUseCase _checkNavigationPrerequisitesUseCase;
  final PerformPreNavigationActionsUseCase _performPreNavigationActionsUseCase;
  final GetWalletStateUseCase _getWalletStateUseCase;

  NavigationService(
    this._navigatorKey,
    this._checkNavigationPrerequisitesUseCase,
    this._performPreNavigationActionsUseCase,
    this._getWalletStateUseCase,
  );

  @override
  FutureOr<void> onDashboardShown() => processQueue();

  /// Process the provided [NavigationRequest], or queue it if the app is in a state where it can't be handled.
  /// Overrides any previously set [NavigationRequest] if this request has to be queued as well.
  Future<void> handleNavigationRequest(NavigationRequest request, {bool queueIfNotReady = false}) async {
    _queuedRequest = null; // It'll get re-queued if it's (still) not ready. Any older request is now considered stale.
    final readyToNavigate = await _checkNavigationPrerequisitesUseCase.invoke(request.navigatePrerequisites);
    if (readyToNavigate) {
      await _navigate(request);
    } else {
      // Resolve the reason as to why we could not navigate
      final state = await _getWalletStateUseCase.invoke();
      final dialogType = _getNavigationBlockedDialogType(state);

      // Reason for not navigating can be explained, inform the user and discard request
      if (dialogType != null) {
        unawaited(showDialog(dialogType, dismissOpenDialogs: true));
        return;
      }

      // No specific reason (e.g. simply waiting for user to unlock). Queue request if needed.
      if (queueIfNotReady) {
        _queuedRequest = request;
        Fimber.d('Not ready to handle $request while in $state. Request queued.');
      }
    }
  }

  /// Returns the type of dialog that should be shown to the user to explain why the navigation was
  /// blocked, content is based on the provided [WalletState]. Returns null if there is no specific
  /// reason to block navigation.
  WalletDialogType? _getNavigationBlockedDialogType(WalletState state) {
    return switch (state) {
      WalletStateUnregistered() || WalletStateEmpty() => .finishSetup,
      WalletStateTransferPossible() => .finishTransferringDestination,
      WalletStateTransferring(role: .source) => WalletDialogType.finishTransferringSource,
      WalletStateTransferring(role: .target) => WalletDialogType.finishTransferringDestination,
      WalletStateInDisclosureFlow() => .finishActiveDisclosureSession,
      WalletStateInIssuanceFlow() => .finishActiveIssuanceSession,
      WalletStateInPinChangeFlow() || WalletStateInPinRecoveryFlow() => .finishRecoverPin,
      WalletStateBlocked() || WalletStateLocked() || WalletStateReady() => null,
    };
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

  /// Attempts to process a previously queued [NavigationRequest].
  ///
  /// This checks if a pending request exists and if its navigation prerequisites
  /// are now met. If so, the request is consumed and executed.
  Future<void> processQueue() async {
    final queuedRequest = _queuedRequest;
    if (queuedRequest == null) return;
    final readyToProcess = await _checkNavigationPrerequisitesUseCase.invoke(queuedRequest.navigatePrerequisites);
    if (readyToProcess) {
      _queuedRequest = null;
      await handleNavigationRequest(queuedRequest);
    } else {
      Fimber.d('Not yet ready to process $_queuedRequest, maintaining queue');
    }
  }

  /// Show the dialog specified by [type]. Useful when caller does not have a valid context.
  Future<void> showDialog(WalletDialogType type, {bool dismissOpenDialogs = false}) async {
    if (dismissOpenDialogs) await _dismissOpenDialogs(); // Dismiss any open dialogs if requested

    final context = _navigatorKey.currentState?.context;
    if (context == null || !context.mounted) return;
    return switch (type) {
      WalletDialogType.idleWarning => IdleWarningDialog.show(context),
      WalletDialogType.resetWallet => ResetWalletDialog.show(context),
      WalletDialogType.scanWithWallet => ScanWithWalletDialog.show(context),
      WalletDialogType.lockedOut => LockedOutDialog.show(context),
      WalletDialogType.moveStopped => MoveStoppedDialog.show(context),
      WalletDialogType.requestNotificationPermission => RequestNotificationPermissionSheet.show(context),
      WalletDialogType.finishActiveDisclosureSession => GenericDialog.showActiveDisclosureSession(context),
      WalletDialogType.finishActiveIssuanceSession => GenericDialog.showActiveIssuanceSession(context),
      WalletDialogType.finishSetup => GenericDialog.showFinishSetup(context),
      WalletDialogType.finishTransferringSource => GenericDialog.showFinishTransferSource(context),
      WalletDialogType.finishTransferringDestination => GenericDialog.showFinishTransferDestination(context),
      WalletDialogType.finishRecoverPin => GenericDialog.showFinishPin(context),
    };
  }

  Future<void> processUpdateNotification(UpdateNotification notification) async {
    final context = _navigatorKey.currentState?.context;
    if (context == null || !context.mounted) return;

    // Show the dialog
    await switch (notification) {
      RecommendUpdateNotification() => UpdateNotificationDialog.show(context),
      WarnUpdateNotification() => UpdateNotificationDialog.show(
        context,
        timeUntilBlocked: notification.timeUntilBlocked,
      ),
    };
  }

  Future<void> _dismissOpenDialogs() async {
    final context = _navigatorKey.currentState?.context;
    if (context != null) await DialogHelper.dismissOpenDialogs(context);
  }
}

enum WalletDialogType {
  idleWarning,
  lockedOut,
  moveStopped,
  resetWallet,
  scanWithWallet,
  requestNotificationPermission,
  finishSetup,
  finishTransferringSource,
  finishTransferringDestination,
  finishActiveDisclosureSession,
  finishActiveIssuanceSession,
  finishRecoverPin,
}
