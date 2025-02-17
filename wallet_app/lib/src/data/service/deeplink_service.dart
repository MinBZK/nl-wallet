import 'dart:async';

import 'package:app_links/app_links.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:rxdart/rxdart.dart';

import '../../domain/usecase/uri/decode_uri_usecase.dart';
import 'app_lifecycle_service.dart';
import 'navigation_service.dart';

@visibleForTesting
const kResumeDebounceDuration = Duration(milliseconds: 100);

class DeeplinkService {
  final AppLinks _appLinks;

  /// Service used to observe the current [AppLifecycleState], so that [Uri]s are
  /// only processed when the app is in the foreground.
  final AppLifecycleService _appLifecycleService;

  final NavigationService _navigationService;

  final DecodeUriUseCase _decodeUriUseCase;

  DeeplinkService(
    this._appLinks,
    this._navigationService,
    this._decodeUriUseCase,
    this._appLifecycleService,
  ) {
    _startObservingAppLinks();
  }

  /// Observe [_appLinks] to process any incoming deeplink. The logic here makes sure the incoming [Uri]s are only
  /// processed when the app is resumed and thus any potential calls to lockWallet have been processed.
  void _startObservingAppLinks() {
    // Note: The [kResumeDebounceDuration] is important, as the apps 'locked' flag is set when the [AppLifecycleState]
    //       changes. Meaning that without the debounceTime the [ObserveWalletLockUseCase] could produce a stale value.
    final initialLinkStream = Stream.fromFuture(_appLinks.getInitialLink()).whereNotNull();
    // This clearController is used to make [allLinksStream] emit null after processing so that the same Uri is not
    // processed twice, which would otherwise happen when the user hides and shows the app.
    final clearController = StreamController<Uri?>();
    final allLinksStream = Rx.merge<Uri?>([initialLinkStream, _appLinks.uriLinkStream, clearController.stream]);
    final debounceUntilResumedStream = CombineLatestStream.combine2(
      allLinksStream,
      _appLifecycleService.observe(),
      (uri, state) => state == AppLifecycleState.resumed ? uri : null,
    ).whereNotNull();
    debounceUntilResumedStream.debounceTime(kResumeDebounceDuration).asyncMap((uri) async {
      clearController.add(null);
      final decodeUriResult = await _decodeUriUseCase.invoke(uri);
      if (decodeUriResult.hasError) throw decodeUriResult.error!;
      return decodeUriResult.value!;
    }).listen(
      (navigationRequest) => _navigationService.handleNavigationRequest(navigationRequest, queueIfNotReady: true),
      onError: (exception) => Fimber.e('Error while processing deeplink', ex: exception),
      cancelOnError: false,
    );
  }
}
