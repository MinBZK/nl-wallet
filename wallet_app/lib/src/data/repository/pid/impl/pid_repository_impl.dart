import 'dart:async';

import 'package:collection/collection.dart';
import 'package:fimber/fimber.dart';
import 'package:rxdart/rxdart.dart';

import '../../../../../bridge_generated.dart';
import '../../../../util/cast_util.dart';
import '../../../../util/mapper/card/card_mapper.dart';
import '../../../../wallet_core/error/core_error.dart';
import '../../../../wallet_core/error/core_error_mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../../../store/active_locale_provider.dart';
import '../pid_repository.dart';

class PidRepositoryImpl extends PidRepository {
  final StreamController<PidIssuanceStatus> _pidIssuanceStatusController = BehaviorSubject();
  final TypedWalletCore _walletCore;
  final CoreErrorMapper _coreErrorMapper;
  final CardMapper _cardMapper;
  final ActiveLocaleProvider _localeProvider;

  PidRepositoryImpl(
    this._walletCore,
    this._coreErrorMapper,
    this._cardMapper,
    this._localeProvider,
  );

  @override
  Future<String> getPidIssuanceUrl() => _walletCore.createPidIssuanceRedirectUri();

  @override
  Future<void> cancelPidIssuance() => _walletCore.cancelPidIssuance();

  @override
  void notifyPidIssuanceStateUpdate(PidIssuanceEvent? event) async {
    /// This doesn't propagate if locale changes, but that is ok for this case, see comment below.
    final activeLocale = await _localeProvider.observe().first;
    event?.when(
      authenticating: () {
        _pidIssuanceStatusController.add(PidIssuanceAuthenticating());
      },
      success: (success) {
        /// Note: Since we don't output a stream here it's not trivial (e.g. with CombineLatestStream)
        /// to make sure the output always contains the desired translations, however in this specific
        /// (pid issuance) case that is irrelevant as the labels are actually ignored. See
        /// [PidAttributeMapper], which combines the attributes provided here and formats to be
        /// displayed with custom labels in a friendlier manner.
        final cards = success.map((card) => _cardMapper.map(card, activeLocale));

        final attributes = cards.map((card) => card.attributes).flattened.toList(growable: false);
        _pidIssuanceStatusController.add(PidIssuanceSuccess(attributes));
        _pidIssuanceStatusController.add(PidIssuanceIdle());
      },
      error: (error) {
        _pidIssuanceStatusController.add(PidIssuanceError(_extractRedirectError(error)));
        _pidIssuanceStatusController.add(PidIssuanceIdle());
      },
    );
    if (event == null) _pidIssuanceStatusController.add(PidIssuanceIdle());
  }

  RedirectError _extractRedirectError(String flutterApiErrorJson) {
    try {
      final coreError = _coreErrorMapper.map(flutterApiErrorJson);
      final redirectUriError = tryCast<CoreRedirectUriError>(coreError);
      return redirectUriError?.redirectError ?? RedirectError.unknown;
    } catch (ex) {
      Fimber.e('Failed to extract RedirectError', ex: ex);
      return RedirectError.unknown;
    }
  }

  @override
  Stream<PidIssuanceStatus> observePidIssuanceStatus() => _pidIssuanceStatusController.stream;

  @override
  Future<WalletInstructionResult> acceptOfferedPid(String pin) => _walletCore.acceptOfferedPid(pin);

  @override
  Future<void> rejectOfferedPid() => _walletCore.rejectOfferedPid();
}
