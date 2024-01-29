import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet/src/data/repository/card/wallet_card_repository.dart';
import 'package:wallet/src/data/repository/history/timeline_attribute_repository.dart';
import 'package:wallet/src/domain/model/timeline/interaction_timeline_attribute.dart';
import 'package:wallet/src/domain/model/timeline/operation_timeline_attribute.dart';
import 'package:wallet/src/domain/model/wallet_card.dart';
import 'package:wallet/src/domain/usecase/card/impl/observe_wallet_card_detail_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/card/observe_wallet_card_detail_usecase.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.dart';

void main() {
  late BehaviorSubject<List<WalletCard>> mockWalletCardsStream;
  late WalletCardRepository mockWalletCardRepository;
  late TimelineAttributeRepository mockTimelineAttributeRepository;

  late ObserveWalletCardDetailUseCase usecase;

  setUp(() {
    mockWalletCardsStream = BehaviorSubject<List<WalletCard>>();
    mockWalletCardRepository = MockWalletCardRepository();
    mockTimelineAttributeRepository = MockTimelineAttributeRepository();

    usecase = ObserveWalletCardDetailUseCaseImpl(
      mockWalletCardRepository,
      mockTimelineAttributeRepository,
    );
  });

  group('invoke', () {
    test('should return card detail on repository stream emit', () async* {
      final WalletCard mockCard = WalletMockData.card;

      when(mockWalletCardRepository.observeWalletCards()).thenAnswer((_) => mockWalletCardsStream);
      when(mockTimelineAttributeRepository.readMostRecentInteraction(mockCard.id, InteractionStatus.success))
          .thenAnswer((_) => Future.value(null));
      when(mockTimelineAttributeRepository.readMostRecentOperation(mockCard.id, OperationStatus.issued))
          .thenAnswer((_) => Future.value(null));

      expectLater(usecase.invoke(WalletMockData.card.id), emits(WalletMockData.cardDetail));

      mockWalletCardsStream.add([WalletMockData.altCard, WalletMockData.card]);

      verify(mockWalletCardRepository.read(mockCard.id)).called(1);
      verify(mockTimelineAttributeRepository.readFiltered(docType: mockCard.docType)).called(1);
    });
  });
}
