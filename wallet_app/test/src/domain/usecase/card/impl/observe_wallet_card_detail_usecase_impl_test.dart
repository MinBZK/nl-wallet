import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet/src/data/repository/card/wallet_card_repository.dart';
import 'package:wallet/src/data/repository/event/wallet_event_repository.dart';
import 'package:wallet/src/domain/model/card/wallet_card.dart';
import 'package:wallet/src/domain/model/event/wallet_event.dart';
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
      final WalletCard mockCard = WalletMockData.card;

      when(mockWalletCardRepository.observeWalletCards()).thenAnswer((_) => mockWalletCardsStream);
      when(mockWalletEventRepository.readMostRecentDisclosureEvent(mockCard.id, EventStatus.success))
          .thenAnswer((_) async => Future.value(null));
      when(mockWalletEventRepository.readMostRecentIssuanceEvent(mockCard.id, EventStatus.success))
          .thenAnswer((_) async => Future.value(null));

      mockWalletCardsStream.add([WalletMockData.altCard, mockCard]);

      final detail = await usecase.invoke(mockCard.id).first;
      expect(detail, WalletMockData.cardDetail);

      verify(mockWalletEventRepository.readMostRecentDisclosureEvent(mockCard.docType, EventStatus.success)).called(1);
      verify(mockWalletEventRepository.readMostRecentIssuanceEvent(mockCard.docType, EventStatus.success)).called(1);
    });
  });
}
