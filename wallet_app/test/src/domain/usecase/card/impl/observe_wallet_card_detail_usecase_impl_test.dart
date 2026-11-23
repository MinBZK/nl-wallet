import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet/src/data/repository/card/wallet_card_repository.dart';
import 'package:wallet/src/data/repository/event/wallet_event_repository.dart';
import 'package:wallet/src/data/repository/wallet/wallet_repository.dart';
import 'package:wallet/src/domain/model/card/wallet_card.dart';
import 'package:wallet/src/domain/model/event/wallet_event.dart';
import 'package:wallet/src/domain/model/wallet_card_detail.dart';
import 'package:wallet/src/domain/usecase/card/impl/observe_wallet_card_detail_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/card/observe_wallet_card_detail_usecase.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.dart';

void main() {
  late BehaviorSubject<List<WalletCard>> mockWalletCardsStream;
  late BehaviorSubject<bool> mockLockedStream;
  late WalletCardRepository mockWalletCardRepository;
  late WalletEventRepository mockWalletEventRepository;
  late WalletRepository mockWalletRepository;

  late ObserveWalletCardDetailUseCase usecase;

  // Create a test card with a known ID for deterministic results
  final WalletCard mockCard = WalletMockData.card.copyWith(attestationId: 'mock');
  // Define the expected enriched card detail with mocked events
  final mockDetail = WalletCardDetail(
    card: mockCard,
    mostRecentIssuance: WalletMockData.issuanceEvent,
    mostRecentSuccessfulDisclosure: WalletMockData.disclosureEvent,
  );

  setUp(() {
    mockWalletCardsStream = BehaviorSubject<List<WalletCard>>();
    mockLockedStream = BehaviorSubject<bool>.seeded(false);
    mockWalletCardRepository = MockWalletCardRepository();
    mockWalletEventRepository = MockWalletEventRepository();
    mockWalletRepository = MockWalletRepository();

    // Card repository returns a stream containing our test card
    when(mockWalletCardRepository.observeWalletCards()).thenAnswer((_) => mockWalletCardsStream);
    when(mockWalletRepository.isLockedStream).thenAnswer((_) => mockLockedStream);

    // Event repository returns predefined events for the card's document type
    when(
      mockWalletEventRepository.readMostRecentDisclosureEvent(mockCard.attestationId!, EventStatus.success),
    ).thenAnswer((_) async => mockDetail.mostRecentSuccessfulDisclosure);
    when(
      mockWalletEventRepository.readMostRecentIssuanceEvent(mockCard.attestationId!, EventStatus.success),
    ).thenAnswer((_) async => mockDetail.mostRecentIssuance);

    usecase = ObserveWalletCardDetailUseCaseImpl(
      mockWalletCardRepository,
      mockWalletEventRepository,
      mockWalletRepository,
    );
  });

  void verifyHistoryRead(int times) {
    verify(
      mockWalletEventRepository.readMostRecentDisclosureEvent(mockCard.attestationId!, EventStatus.success),
    ).called(times);
    verify(
      mockWalletEventRepository.readMostRecentIssuanceEvent(mockCard.attestationId!, EventStatus.success),
    ).called(times);
  }

  void verifyNoHistoryRead() {
    verifyNever(mockWalletEventRepository.readMostRecentDisclosureEvent(mockCard.attestationId!, EventStatus.success));
    verifyNever(mockWalletEventRepository.readMostRecentIssuanceEvent(mockCard.attestationId!, EventStatus.success));
  }

  group('invoke', () {
    test('card detail usecase should enrich card data through event repository', () async {
      // Emit cards through the stream to simulate real-time updates
      mockWalletCardsStream.add([WalletMockData.altCard, mockCard]);

      // Execute the use case and capture the result
      final detail = await usecase.invoke(mockCard.attestationId!).first;

      // Assert the output combines card with correct events
      expect(detail, mockDetail);

      // Verify repositories were consulted for the events
      verifyHistoryRead(1);
    });

    test('history is not read while the wallet is locked', () async {
      mockLockedStream.add(true);
      mockWalletCardsStream.add([mockCard]);

      final details = <WalletCardDetail>[];
      final subscription = usecase.invoke(mockCard.attestationId!).listen(details.add);
      await pumpEventQueue();

      expect(details, isEmpty);
      verifyNoHistoryRead();
      await subscription.cancel();
    });

    test('card detail is refreshed once the wallet is unlocked', () async {
      mockLockedStream.add(true);
      mockWalletCardsStream.add([mockCard]);

      final details = <WalletCardDetail>[];
      final subscription = usecase.invoke(mockCard.attestationId!).listen(details.add);
      await pumpEventQueue();
      mockLockedStream.add(false);
      await pumpEventQueue();

      expect(details, [mockDetail]);
      verifyHistoryRead(1);
      await subscription.cancel();
    });

    test('card updates received while the wallet is locked do not read history', () async {
      mockWalletCardsStream.add([mockCard]);

      final details = <WalletCardDetail>[];
      final subscription = usecase.invoke(mockCard.attestationId!).listen(details.add);
      await pumpEventQueue();
      verifyHistoryRead(1);
      clearInteractions(mockWalletEventRepository);

      mockLockedStream.add(true);
      await pumpEventQueue();
      mockWalletCardsStream.add([WalletMockData.altCard, mockCard]);
      await pumpEventQueue();

      expect(details, [mockDetail]);
      verifyNoHistoryRead();
      await subscription.cancel();
    });

    test('history is not read after the subscription is cancelled', () async {
      mockWalletCardsStream.add([mockCard]);

      final subscription = usecase.invoke(mockCard.attestationId!).listen(null);
      await pumpEventQueue();
      verifyHistoryRead(1);
      clearInteractions(mockWalletEventRepository);

      await subscription.cancel();
      mockWalletCardsStream.add([WalletMockData.altCard, mockCard]);
      await pumpEventQueue();

      verifyNoHistoryRead();
      expect(mockWalletCardsStream.hasListener, isFalse);
    });
  });
}
