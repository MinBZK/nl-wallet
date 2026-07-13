import 'dart:collection';

import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/event/wallet_event.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/event/impl/get_wallet_events_page_usecase_impl.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.dart';

void main() {
  late MockConfigurationRepository mockConfigRepository;
  late MockWalletEventRepository mockEventRepository;
  late GetWalletEventsPageUseCaseImpl useCase;

  late IssuanceEvent pidIssuanceEvent1;
  late IssuanceEvent pidIssuanceEvent2;

  setUp(() {
    mockConfigRepository = MockConfigurationRepository();
    mockEventRepository = MockWalletEventRepository();
    useCase = GetWalletEventsPageUseCaseImpl(mockConfigRepository, mockEventRepository);

    when(
      mockConfigRepository.observeAppConfiguration,
    ).thenAnswer((_) => Stream.value(WalletMockData.flutterAppConfiguration));

    // Two distinct PID IssuanceEvents that are logical duplicates:
    // - different cards (different attestationId) → not == each other
    // - same dateTime/status/eventType → matches() returns true for both
    final pidCard1 = WalletMockData.card.copyWith(
      attestationType: 'urn:eudi:pid:nl:1',
      attestationId: 'pid-1',
      format: .sdJwt,
    );
    final pidCard2 = WalletMockData.card.copyWith(
      attestationType: 'urn:eudi:pid:nl:1',
      attestationId: 'pid-2',
      format: .mdoc,
    );
    final duplicateDateTime = DateTime(2024, 6, 1);
    pidIssuanceEvent1 = IssuanceEvent(
      dateTime: duplicateDateTime,
      status: EventStatus.success,
      card: pidCard1,
      eventType: IssuanceEventType.cardIssued,
    );
    pidIssuanceEvent2 = IssuanceEvent(
      dateTime: duplicateDateTime,
      status: EventStatus.success,
      card: pidCard2,
      eventType: IssuanceEventType.cardIssued,
    );
  });

  group('invoke', () {
    test('fetches and returns the first page when currentPages is empty', () async {
      when(
        mockEventRepository.getEvents(page: 0, pageSize: 25, removeDuplicatePidEvents: false),
      ).thenAnswer((_) async => [WalletMockData.disclosureEvent]);

      final result = await useCase.invoke(page: 0, pageSize: 25, currentPages: SplayTreeMap());

      expect(result.value!.hasNextPage, isFalse);
      expect(result.value!.pages, {
        0: [WalletMockData.disclosureEvent],
      });
    });

    test('hasNextPage is true when the fetched page is full', () async {
      when(
        mockEventRepository.getEvents(page: 0, pageSize: 1, removeDuplicatePidEvents: false),
      ).thenAnswer((_) async => [WalletMockData.disclosureEvent]);

      final result = await useCase.invoke(page: 0, pageSize: 1, currentPages: SplayTreeMap());

      expect(result.value!.hasNextPage, isTrue);
    });

    test('deduplicates duplicate PID events on the first page', () async {
      when(
        mockEventRepository.getEvents(page: 0, pageSize: 25, removeDuplicatePidEvents: false),
      ).thenAnswer((_) async => [WalletMockData.disclosureEvent, pidIssuanceEvent1, pidIssuanceEvent2]);

      final result = await useCase.invoke(page: 0, pageSize: 25, currentPages: SplayTreeMap());

      // pidIssuanceEvent2 is a logical duplicate of pidIssuanceEvent1 (same dateTime/status/eventType);
      // so only the first occurrence (pidIssuanceEvent1) should be kept.
      expect(result.value!.pages, {
        0: [WalletMockData.disclosureEvent, pidIssuanceEvent1],
      });
    });

    test('merges a next page into the existing page window', () async {
      when(
        mockEventRepository.getEvents(page: 1, pageSize: 25, removeDuplicatePidEvents: false),
      ).thenAnswer((_) async => [WalletMockData.issuanceEvent]);

      final currentPages = SplayTreeMap<int, List<WalletEvent>>()..[0] = [WalletMockData.disclosureEvent];
      final result = await useCase.invoke(page: 1, pageSize: 25, currentPages: currentPages);

      expect(result.value!.pages, {
        0: [WalletMockData.disclosureEvent],
        1: [WalletMockData.issuanceEvent],
      });
    });

    test('deduplicates PID events at the boundary between the previous and next page', () async {
      // Page 1 starts with a (higher priority) PID event that is a logical duplicate of pidIssuanceEvent2 on page 0.
      when(
        mockEventRepository.getEvents(page: 1, pageSize: 25, removeDuplicatePidEvents: false),
      ).thenAnswer((_) async => [pidIssuanceEvent1, WalletMockData.issuanceEvent]);

      final currentPages = SplayTreeMap<int, List<WalletEvent>>()
        ..[0] = [WalletMockData.disclosureEvent, pidIssuanceEvent2];
      final result = await useCase.invoke(page: 1, pageSize: 25, currentPages: currentPages);

      expect(result.value!.pages, {
        0: [WalletMockData.disclosureEvent], // pidIssuanceEvent2 is now removed from page 0 (duplicate on page 1)
        1: [pidIssuanceEvent1, WalletMockData.issuanceEvent],
      });
    });

    test('returns error result if repository fails', () async {
      when(
        mockEventRepository.getEvents(page: 0, pageSize: 25, removeDuplicatePidEvents: false),
      ).thenThrow(Exception('Failed'));

      final result = await useCase.invoke(page: 0, pageSize: 25, currentPages: SplayTreeMap());

      expect(result.hasError, isTrue);
      expect(
        result.error,
        isA<GenericError>().having((e) => e.rawMessage, 'rawMessage', contains('Failed')),
      );
    });
  });
}
