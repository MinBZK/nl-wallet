import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/card/format/attestation_format.dart';
import 'package:wallet/src/domain/model/configuration/flutter_app_configuration.dart';
import 'package:wallet/src/domain/model/event/wallet_event.dart';
import 'package:wallet/src/domain/model/pid/pid_attestation.dart';
import 'package:wallet/src/util/mixin/pid_filter_mixin.dart';

import '../../mocks/wallet_mock_data.dart';

class TestPidFilter with PidFilterMixin {
  @override
  final AppConfigurationProvider configProvider;

  TestPidFilter(this.configProvider);
}

void main() {
  late FlutterAppConfiguration config;
  late TestPidFilter pidFilter;

  const pidType1 = 'type1';
  const pidType2 = 'type2';
  const nonPidType = 'non-pid';

  final pidAttestation1 = const PidAttestation(attestationType: pidType1, format: AttestationFormat.mdoc);
  final pidAttestation2 = const PidAttestation(attestationType: pidType2, format: AttestationFormat.sdJwt);

  setUp(() {
    config = WalletMockData.flutterAppConfiguration.copyWith(pidAttestations: [pidAttestation1, pidAttestation2]);
    pidFilter = TestPidFilter(() async => config);
  });

  group('filterDuplicatePidCards', () {
    test('returns all cards when no PID cards are present', () async {
      final cards = [
        WalletMockData.card.copyWith(attestationType: nonPidType),
        WalletMockData.altCard.copyWith(attestationType: nonPidType),
      ];

      final result = await pidFilter.filterDuplicatePidCards(cards);

      expect(result, cards);
    });

    test('returns highest priority PID card and all non-PID cards', () async {
      final pidCard1 = WalletMockData.card.copyWith(attestationType: pidType1, format: AttestationFormat.mdoc);
      final pidCard2 = WalletMockData.card.copyWith(
        attestationId: 'id2',
        attestationType: pidType2,
        format: AttestationFormat.sdJwt,
      );
      final nonPidCard = WalletMockData.card.copyWith(attestationId: 'id3', attestationType: nonPidType);

      final cards = [pidCard2, pidCard1, nonPidCard];

      final result = await pidFilter.filterDuplicatePidCards(cards);

      // pidCard1 has higher priority than pidCard2 in config
      expect(result, containsAllInOrder([pidCard1, nonPidCard]));
      expect(result, isNot(contains(pidCard2)));
    });

    test('returns only the first PID card if multiple of the same priority exist', () async {
      // This is a bit of an edge case, but firstWhereOrNull will pick the first one.
      final pidCard1a = WalletMockData.card.copyWith(
        attestationId: 'id1a',
        attestationType: pidType1,
        format: AttestationFormat.mdoc,
      );
      final pidCard1b = WalletMockData.card.copyWith(
        attestationId: 'id1b',
        attestationType: pidType1,
        format: AttestationFormat.mdoc,
      );

      // Give 1b priority by putting it at the start of the list.
      final cards = [pidCard1b, pidCard1a];

      final result = await pidFilter.filterDuplicatePidCards(cards);

      expect(result, [pidCard1b]);
    });

    test('does not filter if no cards match the config priority list', () async {
      final unknownPidCard = WalletMockData.card.copyWith(
        attestationType: 'unknown-pid',
        format: AttestationFormat.sdJwt,
      );
      final cards = [unknownPidCard];

      final result = await pidFilter.filterDuplicatePidCards(cards);

      expect(result, cards);
    });
  });

  group('filterDuplicatePidEvents', () {
    final dateTime = DateTime(2026, 1, 1);

    test('returns all events when no PID events are present', () async {
      final nonPidCard = WalletMockData.card.copyWith(attestationType: nonPidType);
      final events = [
        WalletEvent.issuance(
          dateTime: dateTime,
          status: EventStatus.success,
          card: nonPidCard,
          eventType: IssuanceEventType.cardIssued,
        ),
        WalletEvent.deletion(
          dateTime: dateTime.add(const Duration(days: 1)),
          status: EventStatus.success,
          card: nonPidCard,
        ),
      ];

      final result = await pidFilter.filterDuplicatePidEvents(events);

      expect(result, events);
    });

    test('filters duplicate PID events, keeping the one with higher priority attestation', () async {
      final pidCard1 = WalletMockData.card.copyWith(attestationType: pidType1, format: AttestationFormat.mdoc);
      final pidCard2 = WalletMockData.card.copyWith(
        attestationId: 'id2',
        attestationType: pidType2,
        format: AttestationFormat.sdJwt,
      );

      final event1 = WalletEvent.issuance(
        dateTime: dateTime,
        status: EventStatus.success,
        card: pidCard1,
        eventType: IssuanceEventType.cardIssued,
      );
      final event2 = WalletEvent.issuance(
        dateTime: dateTime, // Same time, status and eventType
        status: EventStatus.success,
        card: pidCard2,
        eventType: IssuanceEventType.cardIssued,
      );

      final events = [event2, event1];

      final result = await pidFilter.filterDuplicatePidEvents(events);

      // event1 corresponds to pidCard1 which has higher priority
      expect(result, [event1]);
    });

    test('does not filter PID events that are not matches (different actions)', () async {
      final pidCard1 = WalletMockData.card.copyWith(attestationType: pidType1, format: AttestationFormat.sdJwt);
      final pidCard2 = WalletMockData.card.copyWith(
        attestationId: 'id2',
        attestationType: pidType2,
        format: AttestationFormat.sdJwt,
      );

      final event1 = WalletEvent.issuance(
        dateTime: dateTime,
        status: EventStatus.success,
        card: pidCard1,
        eventType: IssuanceEventType.cardIssued,
      );
      final event2 = WalletEvent.issuance(
        dateTime: dateTime.add(const Duration(hours: 1)), // Different time
        status: EventStatus.success,
        card: pidCard2,
        eventType: IssuanceEventType.cardIssued,
      );

      final events = [event1, event2];

      final result = await pidFilter.filterDuplicatePidEvents(events);

      expect(result, containsAllInOrder([event1, event2]));
    });

    test('handles mixed PID and non-PID events correctly', () async {
      final pidCard1 = WalletMockData.card.copyWith(attestationType: pidType1, format: AttestationFormat.mdoc);
      final pidCard2 = WalletMockData.card.copyWith(
        attestationId: 'id2',
        attestationType: pidType2,
        format: AttestationFormat.sdJwt,
      );
      final nonPidCard = WalletMockData.card.copyWith(attestationId: 'id3', attestationType: nonPidType);

      final pidEvent1 = WalletEvent.issuance(
        dateTime: dateTime,
        status: EventStatus.success,
        card: pidCard1,
        eventType: IssuanceEventType.cardIssued,
      );
      final pidEvent2 = WalletEvent.issuance(
        dateTime: dateTime,
        status: EventStatus.success,
        card: pidCard2,
        eventType: IssuanceEventType.cardIssued,
      );
      final nonPidEvent = WalletEvent.issuance(
        dateTime: dateTime,
        status: EventStatus.success,
        card: nonPidCard,
        eventType: IssuanceEventType.cardIssued,
      );

      final events = [pidEvent2, nonPidEvent, pidEvent1];

      final result = await pidFilter.filterDuplicatePidEvents(events);

      // Should keep nonPidEvent and the higher priority pidEvent1
      expect(result, containsAll([pidEvent1, nonPidEvent]));
      expect(result, isNot(contains(pidEvent2)));
    });

    test('filters duplicate PID deletion events', () async {
      final pidCard1 = WalletMockData.card.copyWith(attestationType: pidType1, format: AttestationFormat.mdoc);
      final pidCard2 = WalletMockData.card.copyWith(
        attestationId: 'id2',
        attestationType: pidType2,
        format: AttestationFormat.sdJwt,
      );

      final event1 = WalletEvent.deletion(
        dateTime: dateTime,
        status: EventStatus.success,
        card: pidCard1,
      );
      final event2 = WalletEvent.deletion(
        dateTime: dateTime,
        status: EventStatus.success,
        card: pidCard2,
      );

      final events = [event2, event1];

      final result = await pidFilter.filterDuplicatePidEvents(events);

      expect(result, [event1]);
    });

    test('includes non-card events like disclosure events', () async {
      final pidCard1 = WalletMockData.card.copyWith(attestationType: pidType1, format: AttestationFormat.sdJwt);
      final disclosureEvent = WalletMockData.disclosureEvent;

      final events = [
        WalletEvent.issuance(
          dateTime: dateTime,
          status: EventStatus.success,
          card: pidCard1,
          eventType: IssuanceEventType.cardIssued,
        ),
        disclosureEvent,
      ];

      final result = await pidFilter.filterDuplicatePidEvents(events);

      expect(result, contains(disclosureEvent));
    });
  });
}
