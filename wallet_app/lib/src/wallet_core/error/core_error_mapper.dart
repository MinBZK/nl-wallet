import 'dart:convert';

import '../../util/mapper/mapper.dart';
import 'core_error.dart';
import 'data/core_error_data.dart';
import 'flutter_api_error.dart';

/// Maps a 'FlutterApiErrorJson' [String] to a [CoreError].
class CoreErrorMapper extends Mapper<String, CoreError> {
  @override
  CoreError map(String input) {
    final decodedJson = json.decode(input);
    final flutterApiError = FlutterApiError.fromJson(decodedJson);
    return _mapErrorType(flutterApiError);
  }

  CoreError _mapErrorType(FlutterApiError error) {
    return switch (error.type) {
      FlutterApiErrorType.generic ||
      FlutterApiErrorType.server => CoreGenericError(error.description, data: error.data),
      FlutterApiErrorType.networking => CoreNetworkError(error.description, data: error.data),
      FlutterApiErrorType.walletState => CoreStateError(error.description, data: error.data),
      FlutterApiErrorType.redirectUri => _mapRedirectUriError(error),
      FlutterApiErrorType.hardwareKeyUnsupported => CoreHardwareKeyUnsupportedError(
        error.description,
        data: error.data,
      ),
      FlutterApiErrorType.disclosureSourceMismatch => _mapDisclosureSourceMismatchError(error),
      FlutterApiErrorType.expiredSession => _mapExpiredSessionError(error),
      FlutterApiErrorType.cancelledSession => CoreCancelledSessionError(error.description, data: error.data),
      FlutterApiErrorType.issuer || FlutterApiErrorType.verifier => _mapRelyingPartyError(error),
      FlutterApiErrorType.wrongDigid => CoreWrongDigidError(error.description),
      FlutterApiErrorType.deniedDigid => CoreDeniedDigidError(error.description),
      FlutterApiErrorType.revoked => _mapAccountRevokedError(error),
    };
  }

  CoreError _mapRedirectUriError(FlutterApiError error) {
    final errorData = CoreErrorData.fromJson(error.data ?? {});
    return CoreRedirectUriError(
      error.description,
      data: error.data,
      redirectError: errorData.redirectError ?? .unknown,
    );
  }

  CoreError _mapDisclosureSourceMismatchError(FlutterApiError error) {
    final errorData = CoreErrorData.fromJson(error.data ?? {});
    return CoreDisclosureSourceMismatchError(
      error.description,
      data: error.data,
      isCrossDevice: errorData.sessionType == .crossDevice,
    );
  }

  CoreError _mapExpiredSessionError(FlutterApiError error) {
    final errorData = CoreErrorData.fromJson(error.data ?? {});
    return CoreExpiredSessionError(
      error.description,
      canRetry: errorData.canRetry ?? false,
      data: error.data,
    );
  }

  CoreError _mapRelyingPartyError(FlutterApiError error) {
    final errorData = CoreErrorData.fromJson(error.data ?? {});
    return CoreRelyingPartyError(
      error.description,
      data: error.data,
      organizationName: errorData.mappedOrganizationName,
    );
  }

  CoreError _mapAccountRevokedError(FlutterApiError error) {
    final errorData = CoreErrorData.fromJson(error.data ?? {});
    return CoreAccountRevokedError(
      error.description,
      revocationData: errorData.revocationData ?? RevocationData.unknown(),
      data: error.data,
    );
  }
}
