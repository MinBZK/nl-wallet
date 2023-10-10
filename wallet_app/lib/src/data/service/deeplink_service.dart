import 'dart:async';

import 'package:app_links/app_links.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:rxdart/rxdart.dart';

import '../../../bridge_generated.dart';
import '../../domain/model/navigation/wallet_deeplink.dart';
import '../../domain/usecase/deeplink/decode_deeplink_usecase.dart';
import '../../domain/usecase/navigation/check_navigation_prerequisites_usecase.dart';
import '../../domain/usecase/navigation/perform_pre_navigation_actions_usecase.dart';
import '../../domain/usecase/pid/update_pid_issuance_status_usecase.dart';
import '../../domain/usecase/wallet/observe_wallet_locked_usecase.dart';
import '../../navigation/wallet_routes.dart';
import '../../wallet_core/typed/typed_wallet_core.dart';
import 'app_lifecycle_service.dart';

@visibleForTesting
const kResumeDebounceDuration = Duration(milliseconds: 100);

class DeeplinkService {
  final AppLinks _appLinks;

  /// Key that holds [NavigatorState], used to perform navigation from a non-Widget.
  final GlobalKey<NavigatorState> _navigatorKey;

  /// The [TypedWalletCore], used as a fallback for handling deeplinks
  final TypedWalletCore _walletCore;

  /// A queued [NavigationRequest], when navigation can't be performed (e.g. app
  /// not in a state where it can be handled) maximum 1 [NavigationRequest] is queued
  /// here to be handled when [processQueue] is called.
  NavigationRequest? _queuedRequest;

  /// Subscription of the stream that passes the uri to the wallet_core, making sure the wallet is unlocked
  /// before doing so. If a new deeplink comes in before the current one could be processed (i.e. the wallet was
  /// never unlocked) then the previous deeplink is dismissed by cancelling the old subscription.
  StreamSubscription? _coreDelegationSubscription;

  /// Service used to observe the current [AppLifecycleState], so that [Uri]s are
  /// only processed when the app is in the foreground.
  final AppLifecycleService _appLifecycleService;

  final DecodeDeeplinkUseCase _decodeDeeplinkUseCase;
  final UpdatePidIssuanceStatusUseCase _updatePidIssuanceStatusUseCase;
  final ObserveWalletLockedUseCase _observeWalletLockUseCase;
  final CheckNavigationPrerequisitesUseCase _checkNavigationPrerequisitesUseCase;
  final PerformPreNavigationActionsUseCase _performPreNavigationActionsUseCase;

  DeeplinkService(
    this._appLinks,
    this._navigatorKey,
    this._decodeDeeplinkUseCase,
    this._updatePidIssuanceStatusUseCase,
    this._checkNavigationPrerequisitesUseCase,
    this._performPreNavigationActionsUseCase,
    this._observeWalletLockUseCase,
    this._walletCore,
    this._appLifecycleService,
  ) {
    // Delay the actual processing of the (last seen) [Uri] until the app is resumed.
    // Note: The delay is important, as the apps 'locked' flag is set when the [AppLifecycleState]
    //       changes. Meaning that without the debounceTime the [ObserveWalletLockUseCase] could produce a stale value.
    final resumedStream = _appLifecycleService
        .observe()
        .map((state) => state == AppLifecycleState.resumed)
        .debounceTime(kResumeDebounceDuration);

    Rx.merge([Stream.fromFuture(_appLinks.getInitialAppLink()).whereNotNull(), _appLinks.allUriLinkStream])
        .debounce((uri) => resumedStream)
        .map((uri) {
      final request = _decodeDeeplinkUseCase.invoke(uri);
      if (request != null) return NavigationRequestDeeplink(request, uri);
      return UnknownDeeplink(uri);
    }).listen(_processDeeplink);
  }

  Future<void> _processDeeplink(WalletDeeplink deeplink) async {
    assert(await _appLifecycleService.observe().first == AppLifecycleState.resumed,
        '_processUri should only be called when the app is visible.');
    switch (deeplink) {
      case NavigationRequestDeeplink():
        await _handleNavRequest(deeplink.request);
      case UnknownDeeplink():
        _coreDelegationSubscription?.cancel();
        _coreDelegationSubscription = Stream.value(deeplink.uri)
            .debounce((uri) => _observeWalletLockUseCase.invoke().where((locked) => !locked))
            .listen(_delegateToWalletCore);
    }
  }

  /// Pass the [Uri] to the wallet_core to let it decide how to process it, handling the result.
  Future<void> _delegateToWalletCore(Uri uri) async {
    _walletCore.processUri(uri).listen((event) {
      Fimber.d('wallet_core processUri response: $event');
      event.when(
        pidIssuance: (PidIssuanceEvent event) {
          // We only pass on the [PidIssuanceEvent] here (no navigation) since:
          // - if the app did not cold start the user is already in the correct place
          // - else if the wallet is not yet registered, PidIssuance not yet appropriate and it will be re-initiated later.
          // - else if the wallet is registered but the PID is not yet retrieved, the user will end up in the personalize flow,
          //   the correct state will be rendered because we notify the repository that authentication is in process.
          // - else if the wallet is registered and the PID is available, PidIssuance is no longer relevant.
          _updatePidIssuanceStatusUseCase.invoke(event);
        },
        disclosure: (DisclosureEvent event) => Fimber.d('Received disclosure event: $event'),
        unknownUri: () => Fimber.d('walletCore did not recognize $uri, ignoring.'),
      );
    }, onError: (ex) {
      Fimber.e('processUri() threw an exception while processing $uri', ex: ex);
    }, onDone: () {
      Fimber.d('processUri() stream completed');
    });
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
    await _performPreNavigationActionsUseCase.invoke(request.preNavigationActions);

    _navigatorKey.currentState?.restorablePushNamedAndRemoveUntil(
      request.destination,
      ModalRoute.withName(WalletRoutes.homeRoute),
      arguments: request.argument,
    );
  }

  /// Process any outstanding [NavigationRequest] and consume it if it can be handled.
  Future<void> processQueue() async {
    final queuedRequest = _queuedRequest;
    if (queuedRequest == null) return;
    final readyToNavigate = await _checkNavigationPrerequisitesUseCase.invoke(queuedRequest.navigatePrerequisites);
    if (readyToNavigate) {
      Fimber.d('Prerequisites for $queuedRequest met, clearing queue and executing navigation.');
      _queuedRequest = null;
      await _navigate(queuedRequest);
    } else {
      Fimber.d('Still not ready to navigate based on $queuedRequest, awaiting next call to processQueue() to retry');
    }
  }
}
