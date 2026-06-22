import 'package:collection/collection.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../domain/model/card/wallet_card.dart';
import '../../../domain/model/configuration/flutter_app_configuration.dart';
import '../../../util/extension/pid_attestation_extension.dart';
import '../../../util/mapper/mapper.dart';
import '../../../wallet_core/typed/typed_wallet_core.dart';

/// Mixin providing PID filtering logic based on the app configuration.
mixin PidFilterMixin {
  /// The [TypedWalletCore] used to observe the configuration.
  TypedWalletCore get walletCore;

  /// The mapper used to map the core configuration to [FlutterAppConfiguration].
  Mapper<core.FlutterConfiguration, FlutterAppConfiguration> get flutterAppConfigurationMapper;

  /// Filters [cards] to only maintain the most preferred PID card.
  /// Non-PID cards are always kept.
  Future<List<WalletCard>> filterDuplicatePidCards(List<WalletCard> cards) async {
    final config = await walletCore.observeConfig().map(flutterAppConfigurationMapper.map).first;

    final pidAttestationTypes = config.pidAttestationTypes;

    // Find a matching pid (ordering of [pidAttestations] is leading)
    WalletCard? pidToMaintain;
    for (final pidAttestation in config.pidAttestations) {
      pidToMaintain = cards.firstWhereOrNull(pidAttestation.matches);
      if (pidToMaintain != null) break;
    }

    // Early return in case we don't need to filter
    if (pidToMaintain == null) return cards.toList();

    // Only maintain selected PID + non-PID cards
    return cards.where((card) {
      if (card == pidToMaintain) return true;
      return !pidAttestationTypes.contains(card.attestationType);
    }).toList();
  }
}
