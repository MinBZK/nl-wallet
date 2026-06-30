import 'package:fimber/fimber.dart';

import '../../data/repository/network/network_repository.dart';
import '../../domain/model/result/application_error.dart';
import '../../wallet_core/error/core_error.dart';

extension CoreErrorExtension on CoreError {
  /// Static reference, set on app start, needed to check
  /// internet connection from the [asApplicationError] method.
  static late NetworkRepository networkRepository;

  Future<ApplicationError> asApplicationError() async {
    final error = this; // Assign so we can rely on auto-casting
    return switch (error) {
      CoreGenericError() => GenericError(description, sourceError: error, redirectUrl: returnUrl),
      CoreNetworkError() => _mapNetworkError(error),
      CoreRedirectUriError() => RedirectUriError(redirectError: error.redirectError, sourceError: error),
      CoreHardwareKeyUnsupportedError() => HardwareUnsupportedError(sourceError: error),
      CoreDisclosureSourceMismatchError() => _mapDisclosureSourceMismatchError(error),
      CoreExpiredSessionError() => SessionError(
        state: SessionState.expired,
        canRetry: error.canRetry,
        returnUrl: returnUrl,
        sourceError: error,
      ),
      CoreCancelledSessionError() => SessionError(
        state: SessionState.cancelled,
        returnUrl: returnUrl,
        sourceError: error,
      ),
      CoreCloseProximityDisconnectedError() => CloseProximityDisconnectedError(sourceError: error),
      CoreRelyingPartyError() => _mapRelyingPartyError(error),
      CoreWrongDigidError() => WrongDigidError(sourceError: error),
      CoreDeniedDigidError() => DeniedDigidError(sourceError: error),
      CoreStateError() => _handleStateError(error),
      CoreAccountRevokedError() => AccountRevokedError(
        sourceError: error,
        canRegisterNewAccount: error.canRegisterNewAccount,
      ),
    };
  }

  String? get returnUrl => data?['return_url'];

  Future<ApplicationError> _mapNetworkError(CoreNetworkError error) async {
    final hasInternet = await networkRepository.hasInternet();
    return NetworkError(hasInternet: hasInternet, sourceError: error);
  }

  ApplicationError _mapDisclosureSourceMismatchError(CoreDisclosureSourceMismatchError error) {
    if (error.isCrossDevice) {
      return ExternalScannerError(sourceError: error);
    }
    return GenericError(description, sourceError: error, redirectUrl: returnUrl);
  }

  ApplicationError _mapRelyingPartyError(CoreRelyingPartyError error) {
    return RelyingPartyError(sourceError: error, organizationName: error.organizationName);
  }

  ApplicationError _handleStateError(CoreStateError error) {
    Fimber.e('Invariant violation, mapping to ApplicationStateError', ex: this);
    return ApplicationStateError(sourceError: error);
  }
}
