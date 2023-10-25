import 'dart:async';

import 'package:app_links/app_links.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:rxdart/rxdart.dart';

import '../../domain/usecase/navigation/check_navigation_prerequisites_usecase.dart';
import '../../domain/usecase/navigation/perform_pre_navigation_actions_usecase.dart';
import '../../domain/usecase/uri/decode_uri_usecase.dart';
import '../../navigation/wallet_routes.dart';
import 'app_lifecycle_service.dart';

@visibleForTesting
const kResumeDebounceDuration = Duration(milliseconds: 100);

class DeeplinkService {
  final AppLinks _appLinks;

  /// Key that holds [NavigatorState], used to perform navigation from a non-Widget.
  final GlobalKey<NavigatorState> _navigatorKey;

  /// A queued [NavigationRequest], when navigation can't be performed (e.g. app
  /// not in a state where it can be handled) maximum 1 [NavigationRequest] is queued
  /// here to be handled when [processQueue] is called.
  NavigationRequest? _queuedRequest;

  /// Service used to observe the current [AppLifecycleState], so that [Uri]s are
  /// only processed when the app is in the foreground.
  final AppLifecycleService _appLifecycleService;

  final DecodeUriUseCase _decodeUriUseCase;
  final CheckNavigationPrerequisitesUseCase _checkNavigationPrerequisitesUseCase;
  final PerformPreNavigationActionsUseCase _performPreNavigationActionsUseCase;

  DeeplinkService(
    this._appLinks,
    this._navigatorKey,
    this._decodeUriUseCase,
    this._checkNavigationPrerequisitesUseCase,
    this._performPreNavigationActionsUseCase,
    this._appLifecycleService,
  ) {
    _startObservingAppLinks();
  }

  /// Observe [_appLinks] to process any incoming deeplink. The logic here makes sure the incoming [Uri]s are only
  /// processed when the app is resumed and thus any potential calls to lockWallet have been processed.
  void _startObservingAppLinks() {
    // Note: The [kResumeDebounceDuration] is important, as the apps 'locked' flag is set when the [AppLifecycleState]
    //       changes. Meaning that without the debounceTime the [ObserveWalletLockUseCase] could produce a stale value.
    final initialLinkStream = Stream.fromFuture(_appLinks.getInitialAppLink()).whereNotNull();
    // This clearController is used to make [allLinksStream] emit null after processing so that the same Uri is not
    // processed twice, which would otherwise happen when the user hides and shows the app.
    final clearController = StreamController<Uri?>();
    final allLinksStream = Rx.merge<Uri?>([initialLinkStream, _appLinks.allUriLinkStream, clearController.stream]);
    final debounceUntilResumedStream = CombineLatestStream.combine2(allLinksStream, _appLifecycleService.observe(),
        (uri, state) => state == AppLifecycleState.resumed ? uri : null).whereNotNull();
    debounceUntilResumedStream.debounceTime(kResumeDebounceDuration).asyncMap((uri) async {
      clearController.add(null);
      return await _decodeUriUseCase.invoke(uri);
    }).listen(
      _handleNavRequest,
      onError: (exception) => Fimber.e('Error while processing deeplink', ex: exception),
    );
  }

  /// Process the provided [NavigationRequest], or queue it if the app is in a state where it can't be handled.
  /// Overrides any previously set [NavigationRequest] if this request has to be queued as well.
  Future<void> _handleNavRequest(NavigationRequest request) async {
    final readyToNavigate = await _checkNavigationPrerequisitesUseCase.invoke(request.navigatePrerequisites);
    if (readyToNavigate) {
      await _navigate(request);
    } else {
      Fimber.d('Not yet ready to handle $request, queued and awaiting call to DeeplinkService.processQueue().');
      _queuedRequest = request;
    }
  }

  Future<void> _navigate(NavigationRequest request) async {
    assert(await _checkNavigationPrerequisitesUseCase.invoke(request.navigatePrerequisites),
        'NavigationPreRequisites should have been validated before calling _navigate!');
    await _performPreNavigationActionsUseCase.invoke(request.preNavigationActions);
    switch (request) {
      case PidIssuanceNavigationRequest():
        _navigatorKey.currentState?.pushNamedAndRemoveUntil(
          request.destination,
          ModalRoute.withName(WalletRoutes.splashRoute),
          arguments: request.argument,
        );
      case DisclosureNavigationRequest():
        _navigatorKey.currentState?.pushNamedAndRemoveUntil(
          request.destination,
          ModalRoute.withName(WalletRoutes.homeRoute),
          arguments: request.argument,
        );
      case GenericNavigationRequest():
        _navigatorKey.currentState?.pushNamed(
          request.destination,
          arguments: request.argument,
        );
    }
  }

  /// Process any outstanding [NavigationRequest] and consume it if it can be handled.
  Future<void> processQueue() async {
    final queuedRequest = _queuedRequest;
    _queuedRequest = null;
    if (queuedRequest != null) await _handleNavRequest(queuedRequest);
  }
}
