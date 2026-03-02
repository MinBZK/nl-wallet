import 'package:equatable/equatable.dart';
import 'package:wallet_core/core.dart';

import 'data/redirect/redirect_error.dart';
import 'data/revocation/revocation_data.dart';

export 'data/redirect/redirect_error.dart';
export 'data/revocation/revocation_data.dart';
export 'data/session/session_type.dart';

sealed class CoreError extends Equatable {
  final String? description;
  final Map<String, dynamic>? data;

  const CoreError(this.description, {this.data});

  @override
  List<Object?> get props => [description, data];
}

class CoreGenericError extends CoreError {
  const CoreGenericError(super.description, {super.data});
}

class CoreNetworkError extends CoreError {
  const CoreNetworkError(super.description, {super.data});
}

class CoreStateError extends CoreError {
  const CoreStateError(super.description, {super.data});
}

class CoreRedirectUriError extends CoreError {
  final RedirectError redirectError;

  const CoreRedirectUriError(super.description, {super.data, required this.redirectError});

  @override
  List<Object?> get props => [redirectError, ...super.props];
}

class CoreHardwareKeyUnsupportedError extends CoreError {
  const CoreHardwareKeyUnsupportedError(super.description, {super.data});
}

class CoreDisclosureSourceMismatchError extends CoreError {
  final bool isCrossDevice;

  const CoreDisclosureSourceMismatchError(super.description, {super.data, required this.isCrossDevice});

  @override
  List<Object?> get props => [isCrossDevice, ...super.props];
}

class CoreExpiredSessionError extends CoreError {
  final bool canRetry;

  const CoreExpiredSessionError(super.description, {super.data, required this.canRetry});

  @override
  List<Object?> get props => [canRetry, ...super.props];
}

class CoreCancelledSessionError extends CoreError {
  const CoreCancelledSessionError(super.description, {super.data});
}

class CoreWrongDigidError extends CoreError {
  const CoreWrongDigidError(super.description, {super.data});
}

class CoreDeniedDigidError extends CoreError {
  const CoreDeniedDigidError(super.description, {super.data});
}

class CoreRelyingPartyError extends CoreError {
  final List<LocalizedString>? organizationName;

  CoreRelyingPartyError(super.description, {super.data, this.organizationName})
    : assert(organizationName == null || organizationName.isNotEmpty, 'Do not provide an empty org. name');

  @override
  List<Object?> get props => [organizationName, ...super.props];
}

class CoreAccountRevokedError extends CoreError {
  final RevocationData revocationData;

  bool get canRegisterNewAccount => revocationData.canRegisterNewAccount;

  const CoreAccountRevokedError(
    super.description, {
    super.data,
    required this.revocationData,
  });

  @override
  List<Object?> get props => [revocationData, ...super.props];
}
