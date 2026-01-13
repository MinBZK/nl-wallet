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
    test('ValidityStatus.NotYetValid maps to CardStatusValidSoon', () {
      final validFrom = DateTime(2025, 1, 5);

      final input = _createMockAttestationPresentation(
        revocationStatus: core.RevocationStatus.Valid,
        validityStatus: core.ValidityStatus_NotYetValid(validFrom: validFrom.toIso8601String()),
      );

      expect(mapper.map(input), CardStatusValidSoon(validFrom: validFrom));
    });

    test('ValidityStatus.Valid maps to CardStatusValid', () {
      final validUntil = DateTime(2025, 2, 1);

      final input = _createMockAttestationPresentation(
        revocationStatus: core.RevocationStatus.Valid,
        validityStatus: core.ValidityStatus_Valid(validUntil: validUntil.toIso8601String()),
      );

      expect(mapper.map(input), CardStatusValid(validUntil: validUntil));
    });

    test('ValidityStatus.ExpiresSoon maps to CardStatusExpiresSoon', () {
      final validUntil = DateTime(2025, 1, 5);

      final input = _createMockAttestationPresentation(
        revocationStatus: core.RevocationStatus.Valid,
        validityStatus: core.ValidityStatus_ExpiresSoon(validUntil: validUntil.toIso8601String()),
      );

      expect(mapper.map(input), CardStatusExpiresSoon(validUntil: validUntil));
    });

    test('ValidityStatus.Expired maps to CardStatusExpired', () {
      final validUntil = DateTime(2024, 12, 25);

      final input = _createMockAttestationPresentation(
        revocationStatus: core.RevocationStatus.Valid,
        validityStatus: core.ValidityStatus_Expired(validUntil: validUntil.toIso8601String()),
      );

      expect(mapper.map(input), CardStatusExpired(validUntil: validUntil));
    });

    test('Revoked revocationStatus should return CardStatusRevoked', () {
      final input = _createMockAttestationPresentation(
        revocationStatus: core.RevocationStatus.Revoked,
        validityStatus: const core.ValidityStatus_Valid(validUntil: null),
      );

      expect(mapper.map(input), const CardStatusRevoked());
    });

    test('Corrupted revocationStatus should return CardStatusCorrupted', () {
      final input = _createMockAttestationPresentation(
        revocationStatus: core.RevocationStatus.Corrupted,
        validityStatus: const core.ValidityStatus_Valid(validUntil: null),
      );

      expect(mapper.map(input), const CardStatusCorrupted());
    });

    test('Undetermined revocationStatus should return CardStatusUndetermined', () {
      final input = _createMockAttestationPresentation(
        revocationStatus: core.RevocationStatus.Undetermined,
        validityStatus: const core.ValidityStatus_Valid(validUntil: null),
      );

      expect(mapper.map(input), const CardStatusUndetermined());
    });

    test('Null revocationStatus should return CardStatusValid', () {
      withClock(Clock.fixed(DateTime(2025, 1, 1)), () {
        final input = _createMockAttestationPresentation(
          revocationStatus: null,
          validityStatus: core.ValidityStatus_Valid(validUntil: DateTime(2025, 1, 1).toIso8601String()),
        );

        expect(mapper.map(input), CardStatusValid(validUntil: DateTime(2025, 1, 1)));
      });
    });
  });
}

core.AttestationPresentation _createMockAttestationPresentation({
  core.RevocationStatus? revocationStatus,
  required core.ValidityStatus validityStatus,
}) {
  return core.AttestationPresentation(
    identity: const core.AttestationIdentity_Fixed(id: 'id'),
    attestationType: 'com.example.attestation',
    displayMetadata: [CoreMockData.enDisplayMetadata, CoreMockData.nlDisplayMetadata],
    issuer: CoreMockData.organization,
    attributes: [CoreMockData.attestationAttributeName],
    revocationStatus: revocationStatus,
    validityStatus: validityStatus,
  );
}
