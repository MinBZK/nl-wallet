import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet/src/data/repository/card/wallet_card_repository.dart';
import 'package:wallet/src/data/repository/event/wallet_event_repository.dart';
import 'package:wallet/src/domain/model/event/wallet_event.dart';
import 'package:wallet/src/domain/model/wallet_card.dart';
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
    test('should return card detail on repository stream emit', () async* {
      final WalletCard mockCard = WalletMockData.card;

      when(mockWalletCardRepository.observeWalletCards()).thenAnswer((_) => mockWalletCardsStream);
      when(mockWalletEventRepository.readMostRecentDisclosureEvent(mockCard.id, EventStatus.success))
          .thenAnswer((_) => Future.value(null));
      when(mockWalletEventRepository.readMostRecentIssuanceEvent(mockCard.id, EventStatus.success))
          .thenAnswer((_) => Future.value(null));

      expectLater(usecase.invoke(WalletMockData.card.id), emits(WalletMockData.cardDetail));

      mockWalletCardsStream.add([WalletMockData.altCard, WalletMockData.card]);

      verify(mockWalletCardRepository.read(mockCard.id)).called(1);
      verify(mockWalletEventRepository.readMostRecentDisclosureEvent(mockCard.id, EventStatus.success)).called(1);
      verify(mockWalletEventRepository.readMostRecentIssuanceEvent(mockCard.id, EventStatus.success)).called(1);
    });
  });
}
