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
    // A stream that emits the latest link from either the initial link or subsequent links.
    final clearController = StreamController<Uri?>();
    final uriStream = Rx.merge<Uri?>([
      Stream.fromFuture(_appLinks.getInitialLink()),
      _appLinks.uriLinkStream,
      clearController.stream,
    ]);

    // Combine the latest link with the latest lifecycle state, so we only process when app is resumed.
    CombineLatestStream.combine2(uriStream, _appLifecycleService.observe(), (uri, state) => (uri: uri, state: state))
        .where((data) => data.state == .resumed) // Only emit when .resumed
        .map((data) => data.uri) // We only care about the uri
        .distinct() // Don't process uris twice
        .whereNotNull()
        .debounceTime(kResumeDebounceDuration) // Debounce (chance to process background auto-lock)
        .asyncMap((uri) async {
          clearController.add(null); // Reset uriStream so that a deeplink can be processed again later
          final decodeUriResult = await _decodeUriUseCase.invoke(uri);
          if (decodeUriResult.hasError) throw decodeUriResult.error!;
          return decodeUriResult.value!;
        })
        .listen(
          (navRequest) => _navigationService.handleNavigationRequest(navRequest, queueIfNotReady: true),
          onError: (exception) => Fimber.e('Error while processing deeplink', ex: exception),
          cancelOnError: false,
        );
  }
}
