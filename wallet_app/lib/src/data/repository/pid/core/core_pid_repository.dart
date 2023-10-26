import 'dart:async';

import 'package:collection/collection.dart';
import 'package:rxdart/rxdart.dart';

import '../../../../../bridge_generated.dart';
import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../domain/model/wallet_card.dart';
import '../../../../util/mapper/locale_mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../../../store/active_locale_provider.dart';
import '../pid_repository.dart';

class CorePidRepository extends PidRepository {
  final TypedWalletCore _walletCore;
  final LocaleMapper<Card, WalletCard> _cardMapper;
  final ActiveLocaleProvider _localeProvider;

  CorePidRepository(
    this._walletCore,
    this._cardMapper,
    this._localeProvider,
  );

  @override
  Future<String> getPidIssuanceUrl() => _walletCore.createPidIssuanceRedirectUri();

  @override
  Stream<List<DataAttribute>> continuePidIssuance(Uri uri) {
    return CombineLatestStream.combine2(
        Stream.fromFuture(_walletCore.continuePidIssuance(uri)),
        _localeProvider.observe(),
        (previewCards, locale) => previewCards
            .map((card) => _cardMapper.map(locale, card))
            .map((card) => card.attributes)
            .flattened
            .toList());
  }

  @override
  Future<void> cancelPidIssuance() => _walletCore.cancelPidIssuance();

  @override
  Future<WalletInstructionResult> acceptOfferedPid(String pin) => _walletCore.acceptOfferedPid(pin);

  @override
  Future<void> rejectOfferedPid() => _walletCore.rejectOfferedPid();
}
