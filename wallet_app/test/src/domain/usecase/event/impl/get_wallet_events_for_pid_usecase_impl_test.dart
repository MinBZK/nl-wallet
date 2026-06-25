import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/card/format/attestation_format.dart';
import 'package:wallet/src/domain/model/event/wallet_event.dart';
import 'package:wallet/src/domain/model/pid/pid_attestation.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/event/impl/get_wallet_events_for_pid_usecase_impl.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.dart';

void main() {
  late MockConfigurationRepository mockConfigRepository;
  late MockWalletEventRepository mockEventRepository;
  late MockWalletCardRepository mockCardRepository;
  late GetWalletEventsForPidUseCaseImpl useCase;

  setUp(() {
    mockConfigRepository = MockConfigurationRepository();
    mockEventRepository = MockWalletEventRepository();
    mockCardRepository = MockWalletCardRepository();
    useCase = GetWalletEventsForPidUseCaseImpl(
      mockConfigRepository,
      mockEventRepository,
      mockCardRepository,
    );
  });

  group('invoke', () {
    test('returns filtered and sorted PID events', () async {
      const pidType = 'urn:eudi:pid:nl:1';
      final config = WalletMockData.flutterAppConfiguration.copyWith(
        pidAttestations: [
          const PidAttestation(attestationType: pidType, format: AttestationFormat.sdJwt),
          const PidAttestation(attestationType: pidType, format: AttestationFormat.mdoc),
        ],
      );

      final pidCard1 = WalletMockData.card.copyWith(
        attestationId: 'id1',
        attestationType: pidType,
        format: AttestationFormat.sdJwt,
      );
      final pidCard2 = WalletMockData.card.copyWith(
        attestationId: 'id2',
        attestationType: pidType,
        format: AttestationFormat.mdoc,
      );
      final nonPidCard = WalletMockData.card.copyWith(attestationId: 'id3', attestationType: 'other');

      final event1 = WalletEvent.issuance(
        dateTime: DateTime(2024, 1, 1),
        status: EventStatus.success,
        card: pidCard1,
        eventType: IssuanceEventType.cardIssued,
      );
      final event2 = WalletEvent.issuance(
        dateTime: DateTime(2024, 1, 1),
        status: EventStatus.success,
        card: pidCard2,
        eventType: IssuanceEventType.cardIssued,
      );
      final recentEvent = WalletEvent.deletion(
        dateTime: DateTime(2024, 1, 2),
        status: EventStatus.success,
        card: pidCard1,
      );

      when(mockConfigRepository.observeAppConfiguration).thenAnswer((_) => Stream.value(config));
      when(
        mockCardRepository.readAll(filterDuplicatePids: false),
      ).thenAnswer((_) async => [pidCard1, pidCard2, nonPidCard]);
      when(mockEventRepository.getEventsForCard('id1')).thenAnswer((_) async => [event1, recentEvent]);
      when(mockEventRepository.getEventsForCard('id2')).thenAnswer((_) async => [event2]);

      final result = await useCase.invoke();

      expect(result.value, hasLength(2));
      // sorted by date DESC (newest first)
      expect(result.value![0], recentEvent);
      // event1 and event2 are considered duplicates, event2 (lowest prio PID) should be filtered out.
      expect(result.value![1], event1);

      verify(mockEventRepository.getEventsForCard('id1')).called(1);
      verify(mockEventRepository.getEventsForCard('id2')).called(1);
      verifyNever(mockEventRepository.getEventsForCard('id3'));
    });

    test('returns empty list if no PID cards exist', () async {
      final config = WalletMockData.flutterAppConfiguration;
      final nonPidCard = WalletMockData.card.copyWith(attestationType: 'other');

      when(mockConfigRepository.observeAppConfiguration).thenAnswer((_) => Stream.value(config));
      when(mockCardRepository.readAll(filterDuplicatePids: false)).thenAnswer((_) async => [nonPidCard]);

      final result = await useCase.invoke();

      expect(result.value, isEmpty);
      verifyNever(mockEventRepository.getEventsForCard(any));
    });

    test('returns error result if repository fails', () async {
      final exception = Exception('Failed');
      when(mockConfigRepository.observeAppConfiguration).thenAnswer((_) => Stream.error(exception));

      final result = await useCase.invoke();

      expect(result.hasError, isTrue);
      expect(
        result.error,
        isA<GenericError>().having((e) => e.rawMessage, 'rawMessage', contains('Failed')),
      );
    });
  });
}
