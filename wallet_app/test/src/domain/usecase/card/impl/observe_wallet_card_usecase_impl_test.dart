import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet/src/data/repository/card/wallet_card_repository.dart';
import 'package:wallet/src/domain/model/card/wallet_card.dart';
import 'package:wallet/src/domain/usecase/card/impl/observe_wallet_card_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/card/observe_wallet_card_usecase.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.dart';

void main() {
  late BehaviorSubject<List<WalletCard>> mockWalletCardsStream;
  late WalletCardRepository mockWalletCardRepository;

  late ObserveWalletCardUseCase usecase;

  setUp(() {
    mockWalletCardsStream = BehaviorSubject<List<WalletCard>>();
    mockWalletCardRepository = MockWalletCardRepository();

    usecase = ObserveWalletCardUseCaseImpl(mockWalletCardRepository);
  });

  group('invoke', () {
    test('should return a list of cards on repository stream emit', () {
      when(mockWalletCardRepository.observeWalletCards()).thenAnswer((_) => mockWalletCardsStream);

      expectLater(usecase.invoke(WalletMockData.card.id), emits(WalletMockData.card));

      mockWalletCardsStream.add([WalletMockData.altCard, WalletMockData.card]);
    });
  });
}
