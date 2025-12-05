import 'package:wallet_core/core.dart' as core;

import '../../../../domain/model/card/status/card_status.dart';
import '../../mapper.dart';

/// Maps a [core.RevocationStatus] to a [CardStatus].
class CardStatusMapper extends Mapper<core.RevocationStatus?, CardStatus> {
  CardStatusMapper();

  @override
  CardStatus map(core.RevocationStatus? input) {
    final revocationStatus = input;
    if (revocationStatus == null) return CardStatus.unknown;

    // TODO(Daan): Implement validSoon, expiresSoon & expired mapping once Core logic is implemented in [PVW-5161];
    return switch (revocationStatus) {
      core.RevocationStatus.Valid => CardStatus.valid,
      core.RevocationStatus.Revoked => CardStatus.revoked,
      core.RevocationStatus.Corrupted => CardStatus.corrupted,
      core.RevocationStatus.Undetermined => CardStatus.unknown,
    };
  }
}
