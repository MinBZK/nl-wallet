import '../card/wallet_card.dart';
import '../document.dart';
import '../organization.dart';
import '../policy/policy.dart';

sealed class StartSignResult {
  final Organization relyingParty;
  final Organization trustProvider;
  final Policy policy;
  final Document document;

  const StartSignResult({
    required this.relyingParty,
    required this.trustProvider,
    required this.policy,
    required this.document,
  });
}

class StartSignReadyToSign extends StartSignResult {
  final List<WalletCard> requestedCards;

  StartSignReadyToSign({
    required super.relyingParty,
    required super.trustProvider,
    required super.policy,
    required super.document,
    required this.requestedCards,
  });
}

/// Not yet implemented for the mock, since all mock usecases
/// only rely on data in the PID, which is always available.
// class StartSignMissingAttributes extends StartSignResult {
//   final List<MissingAttribute> missingAttributes;
//
//   StartSignMissingAttributes({
//     required super.relyingParty,
//     required super.trustProvider,
//     required super.policy,
//     required super.document,
//     required this.missingAttributes,
//   });
// }
