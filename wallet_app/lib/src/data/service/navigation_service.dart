import 'dart:async';

import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';

import '../../domain/usecase/navigation/check_navigation_prerequisites_usecase.dart';
import '../../domain/usecase/navigation/perform_pre_navigation_actions_usecase.dart';
import '../../navigation/wallet_routes.dart';

class NavigationService {
  /// Key that holds [NavigatorState], used to perform navigation from a non-Widget.
  final GlobalKey<NavigatorState> _navigatorKey;

  /// A queued [NavigationRequest], when navigation can't be performed (e.g. app
  /// not in a state where it can be handled) maximum 1 [NavigationRequest] is queued
  /// here to be handled when [processQueue] is called.
  NavigationRequest? _queuedRequest;

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
      case DisclosureNavigationRequest():
        await _navigatorKey.currentState?.pushNamedAndRemoveUntil(
          request.destination,
          ModalRoute.withName(WalletRoutes.dashboardRoute),
          arguments: request.argument,
        );
      case IssuanceNavigationRequest():
        await _navigatorKey.currentState?.pushNamedAndRemoveUntil(
          request.destination,
          ModalRoute.withName(WalletRoutes.dashboardRoute),
          arguments: request.argument,
        );
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
}
