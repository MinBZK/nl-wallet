import 'package:fimber/fimber.dart';

import '../../data/repository/network/network_repository.dart';
import '../../domain/model/attribute/attribute.dart';
import '../../domain/model/result/application_error.dart';
import '../../wallet_core/error/core_error.dart';
import '../mapper/card/attribute/localized_labels_mapper.dart';

extension CoreErrorExtension on CoreError {
  /// Static reference, set on app start, needed to check
  /// internet connection from the [asApplicationError] method.
  static late NetworkRepository networkRepository;

  Future<ApplicationError> asApplicationError() async {
    final error = this; // Assign so we can rely on auto-casting
    switch (error) {
      case CoreGenericError():
        return GenericError(description, sourceError: error, redirectUrl: returnUrl);
      case CoreNetworkError():
        final hasInternet = await networkRepository.hasInternet();
        return NetworkError(hasInternet: hasInternet, sourceError: error);
      case CoreRedirectUriError():
        return RedirectUriError(redirectError: error.redirectError, sourceError: error);
      case CoreHardwareKeyUnsupportedError():
        return HardwareUnsupportedError(sourceError: error);
      case CoreDisclosureSourceMismatchError():
        if (error.isCrossDevice) return ExternalScannerError(sourceError: error);
        return GenericError(description, sourceError: error, redirectUrl: returnUrl);
      case CoreExpiredSessionError():
        return SessionError(
          state: SessionState.expired,
          canRetry: error.canRetry,
          returnUrl: returnUrl,
          sourceError: error,
        );
      case CoreCancelledSessionError():
        return SessionError(state: SessionState.cancelled, returnUrl: returnUrl, sourceError: error);
      case CoreRelyingPartyError():
        LocalizedText? organizationName;
        if (error.organizationName != null) organizationName = LocalizedLabelsMapper().map(error.organizationName!);
        return RelyingPartyError(sourceError: error, organizationName: organizationName);
      case CoreStateError():
        Fimber.e('StateError detected!', ex: this);
        throw StateError(toString());
    }
  }

  String? get returnUrl => data?['return_url'];
}
