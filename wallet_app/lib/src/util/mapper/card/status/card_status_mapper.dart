import 'package:wallet_core/core.dart' as core;

import '../../../../domain/model/card/status/card_status.dart';
import '../../mapper.dart';

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

    final validityStatus = input.validityStatus;
    return switch (validityStatus) {
      core.ValidityStatus_NotYetValid() => CardStatusValidSoon(
        validFrom: DateTime.parse(validityStatus.validFrom).toLocal(),
      ),
      core.ValidityStatus_Valid() => CardStatusValid(
        validUntil: validityStatus.validUntil == null ? null : DateTime.parse(validityStatus.validUntil!).toLocal(),
      ),
      core.ValidityStatus_ExpiresSoon() => CardStatusExpiresSoon(
        validUntil: DateTime.parse(validityStatus.validUntil).toLocal(),
      ),
      core.ValidityStatus_Expired() => CardStatusExpired(
        validUntil: DateTime.parse(validityStatus.validUntil).toLocal(),
      ),
    };
  }
}
