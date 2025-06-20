import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet/src/data/repository/card/wallet_card_repository.dart';
import 'package:wallet/src/data/repository/event/wallet_event_repository.dart';
import 'package:wallet/src/domain/model/card/wallet_card.dart';
import 'package:wallet/src/domain/model/event/wallet_event.dart';
import 'package:wallet/src/domain/model/wallet_card_detail.dart';
import 'package:wallet/src/domain/usecase/card/impl/observe_wallet_card_detail_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/card/observe_wallet_card_detail_usecase.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.dart';

void main() {
  late BehaviorSubject<List<WalletCard>> mockWalletCardsStream;
  late WalletCardRepository mockWalletCardRepository;
  late WalletEventRepository mockWalletEventRepository;

  late ObserveWalletCardDetailUseCase usecase;

  setUp(() {
    mockWalletCardsStream = BehaviorSubject<List<WalletCard>>();
    mockWalletCardRepository = MockWalletCardRepository();
    mockWalletEventRepository = MockWalletEventRepository();

    usecase = ObserveWalletCardDetailUseCaseImpl(
      mockWalletCardRepository,
      mockWalletEventRepository,
    );
  });

  group('invoke', () {
    test('card detail usecase should enrich card data through event repository', () async {
      // Create a test card with a known ID for deterministic results
      final WalletCard mockCard = WalletMockData.card.copyWith(id: () => 'mock');
      // Define the expected enriched card detail with mocked events
      final mockDetail = WalletCardDetail(
        card: mockCard,
        mostRecentIssuance: WalletMockData.issuanceEvent,
        mostRecentSuccessfulDisclosure: WalletMockData.disclosureEvent,
      );

      // Card repository returns a stream containing our test card
      when(mockWalletCardRepository.observeWalletCards()).thenAnswer((_) => mockWalletCardsStream);

      // Event repository returns predefined events for the card's document type
      when(
        mockWalletEventRepository.readMostRecentDisclosureEvent(mockCard.id!, EventStatus.success),
      ).thenAnswer((_) async => mockDetail.mostRecentSuccessfulDisclosure);
      when(
        mockWalletEventRepository.readMostRecentIssuanceEvent(mockCard.id!, EventStatus.success),
      ).thenAnswer((_) async => mockDetail.mostRecentIssuance);

      // Emit cards through the stream to simulate real-time updates
      mockWalletCardsStream.add([WalletMockData.altCard, mockCard]);

      // Execute the use case and capture the result
      final detail = await usecase.invoke(mockCard.id!).first;

      // Assert the output combines card with correct events
      expect(detail, mockDetail);

      // Verify repositories were consulted for the events
      verify(mockWalletEventRepository.readMostRecentDisclosureEvent(mockCard.id!, EventStatus.success)).called(1);
      verify(mockWalletEventRepository.readMostRecentIssuanceEvent(mockCard.id!, EventStatus.success)).called(1);
    });
  });
}
