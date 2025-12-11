import 'package:clock/clock.dart';
import 'package:flutter/foundation.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../../domain/model/card/status/card_status.dart';
import '../../mapper.dart';

/// Threshold in days to consider a card as "expires soon".
@visibleForTesting
const kCardExpiresSoonThresholdInDays = 14;

/// Maps a [core.AttestationPresentation] to a [CardStatus].
class CardStatusMapper extends Mapper<core.AttestationPresentation, CardStatus> {
  CardStatusMapper();

  @override
  CardStatus map(core.AttestationPresentation input) {
    return switch (input.revocationStatus) {
      /// When revocation status is null, we assume the status is valid;
      /// This occurs when the attestation is issued and the revocation status is not checked yet.
      null => _mapValidRevocationStatus(input),
      core.RevocationStatus.Valid => _mapValidRevocationStatus(input),
      core.RevocationStatus.Revoked => const CardStatusRevoked(),
      core.RevocationStatus.Corrupted => const CardStatusCorrupted(),
      core.RevocationStatus.Undetermined => const CardStatusUndetermined(),
    };
  }

  CardStatus _mapValidRevocationStatus(core.AttestationPresentation input) {
    final revocationStatus = input.revocationStatus;
    assert(
      revocationStatus == null || revocationStatus == core.RevocationStatus.Valid,
      'Invalid revocation status, expecting: "null" or "Valid" but found: "$revocationStatus".',
    );

    final validFromString = input.validityWindow.validFrom;
    final validUntilString = input.validityWindow.validUntil;
    final validFrom = validFromString != null ? DateTime.parse(validFromString).toLocal() : null;
    final validUntil = validUntilString != null ? DateTime.parse(validUntilString).toLocal() : null;
    final now = clock.now();

    // Check if card is not yet valid but will be valid soon
    if (validFrom?.isAfter(now) ?? false) {
      return CardStatusValidSoon(validFrom: validFrom!);
    }
    // Check if card is expired
    if (validUntil?.isBefore(now) ?? false) {
      return CardStatusExpired(validUntil: validUntil!);
    }
    // Check if card expires soon
    if (validUntil != null && validUntil.difference(now).inDays <= kCardExpiresSoonThresholdInDays) {
      return CardStatusExpiresSoon(validUntil: validUntil);
    }
    // All checks passed
    return CardStatusValid(validUntil: validUntil);
  }
}
