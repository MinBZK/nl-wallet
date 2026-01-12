import 'package:clock/clock.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/card/status/card_status.dart';
import 'package:wallet/src/util/mapper/card/status/card_status_mapper.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../../mocks/core_mock_data.dart';

void main() {
  late CardStatusMapper mapper;

  setUp(() {
    mapper = CardStatusMapper();
  });

  group('map', () {
    test('Valid should return CardStatus.validSoon when validFrom is in the future', () {
      withClock(Clock.fixed(DateTime(2025, 1, 1)), () {
        final validFrom = DateTime(2025, 1, 5);
        final input = _createMockAttestationPresentation(
          revocationStatus: core.RevocationStatus.Valid,
          validFrom: validFrom.toIso8601String(),
        );

        expect(mapper.map(input), CardStatusValidSoon(validFrom: validFrom));
      });
    });

    test('Valid should return CardStatus.valid when no validUntil is provided', () {
      withClock(Clock.fixed(DateTime(2025, 1, 1)), () {
        final input = _createMockAttestationPresentation(
          revocationStatus: core.RevocationStatus.Valid,
          validFrom: DateTime(2024, 12, 1).toIso8601String(),
          validUntil: null,
        );

        expect(mapper.map(input), const CardStatusValid(validUntil: null));
      });
    });

    test('Valid should return CardStatus.valid when validUntil is far in the future', () {
      withClock(Clock.fixed(DateTime(2025, 1, 1)), () {
        final validUntil = DateTime(2025, 2, 1);
        final input = _createMockAttestationPresentation(
          revocationStatus: core.RevocationStatus.Valid,
          validFrom: DateTime(2024, 12, 1).toIso8601String(),
          validUntil: validUntil.toIso8601String(),
        );

        final CardStatus actual = mapper.map(input);

        expect(actual, CardStatusValid(validUntil: validUntil));
      });
    });

    test('Valid should return CardStatus.expiresSoon when validUntil is within threshold', () {
      withClock(Clock.fixed(DateTime(2025, 1, 1)), () {
        final validUntil = DateTime(2025, 1, 5); // 4 days from now
        final input = _createMockAttestationPresentation(
          revocationStatus: core.RevocationStatus.Valid,
          validFrom: DateTime(2024, 12, 1).toIso8601String(),
          validUntil: validUntil.toIso8601String(),
        );

        final CardStatus actual = mapper.map(input);

        expect(actual, CardStatusExpiresSoon(validUntil: validUntil));
      });
    });

    test('Valid should return CardStatus.expired when validUntil is in the past', () {
      withClock(Clock.fixed(DateTime(2025, 1, 1)), () {
        final validUntil = DateTime(2024, 12, 25); // Past date
        final input = _createMockAttestationPresentation(
          revocationStatus: core.RevocationStatus.Valid,
          validFrom: DateTime(2024, 12, 1).toIso8601String(),
          validUntil: validUntil.toIso8601String(),
        );

        expect(mapper.map(input), CardStatusExpired(validUntil: validUntil));
      });
    });

    test('Valid should return CardStatus.expiresSoon when validUntil is exactly at threshold', () {
      withClock(Clock.fixed(DateTime(2025, 1, 1)), () {
        final validUntil = DateTime(2025, 1, kCardExpiresSoonThresholdInDays);
        final input = _createMockAttestationPresentation(
          revocationStatus: core.RevocationStatus.Valid,
          validFrom: DateTime(2024, 12, 1).toIso8601String(),
          validUntil: validUntil.toIso8601String(),
        );

        expect(mapper.map(input), CardStatusExpiresSoon(validUntil: validUntil));
      });
    });

    test('Revoked revocationStatus should return CardStatusRevoked', () {
      final input = _createMockAttestationPresentation(revocationStatus: core.RevocationStatus.Revoked);

      expect(mapper.map(input), const CardStatusRevoked());
    });

    test('Corrupted revocationStatus should return CardStatusCorrupted', () {
      final input = _createMockAttestationPresentation(revocationStatus: core.RevocationStatus.Corrupted);

      expect(mapper.map(input), const CardStatusCorrupted());
    });

    test('Undetermined revocationStatus should return CardStatusUndetermined', () {
      final input = _createMockAttestationPresentation(revocationStatus: core.RevocationStatus.Undetermined);

      expect(mapper.map(input), const CardStatusUndetermined());
    });

    test('Null revocationStatus should return CardStatusValid', () {
      final input = _createMockAttestationPresentation(revocationStatus: null);

      expect(mapper.map(input), const CardStatusValid(validUntil: null));
    });
  });
}

core.AttestationPresentation _createMockAttestationPresentation({
  core.RevocationStatus? revocationStatus,
  String? validFrom,
  String? validUntil,
}) {
  return core.AttestationPresentation(
    identity: const core.AttestationIdentity_Fixed(id: 'id'),
    attestationType: 'com.example.attestation',
    displayMetadata: [CoreMockData.enDisplayMetadata, CoreMockData.nlDisplayMetadata],
    issuer: CoreMockData.organization,
    attributes: [CoreMockData.attestationAttributeName],
    revocationStatus: revocationStatus,
    validityStatus: core.ValidityStatus.Valid,
    validityWindow: core.ValidityWindow(
      validFrom: validFrom,
      validUntil: validUntil,
    ),
  );
}
