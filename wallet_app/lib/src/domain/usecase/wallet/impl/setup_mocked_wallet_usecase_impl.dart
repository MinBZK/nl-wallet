import '../../../../data/repository/card/timeline_attribute_repository.dart';
import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../../data/repository/issuance/issuance_response_repository.dart';
import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../../model/issuance_response.dart';
import '../../../model/timeline/operation_timeline_attribute.dart';
import '../setup_mocked_wallet_usecase.dart';

class SetupMockedWalletUseCaseImpl implements SetupMockedWalletUseCase {
  final WalletRepository walletRepository;
  final WalletCardRepository walletCardRepository;
  final IssuanceResponseRepository issuanceResponseRepository;
  final TimelineAttributeRepository timelineAttributeRepository;

  SetupMockedWalletUseCaseImpl(
    this.walletRepository,
    this.walletCardRepository,
    this.issuanceResponseRepository,
    this.timelineAttributeRepository,
  );

  @override
  Future<void> invoke() async {
    // Create wallet
    await walletRepository.createWallet('000000');
    walletRepository.unlockWallet('000000');

    // Add cards + history
    const cardIds = ['PID_1', 'DRIVING_LICENSE'];
    for (String cardId in cardIds) {
      // Add card
      final IssuanceResponse issuanceResponse = await issuanceResponseRepository.read(cardId);
      final card = issuanceResponse.cards.first;
      walletCardRepository.create(card);

      // Add history
      timelineAttributeRepository.create(
        OperationTimelineAttribute(
          status: OperationStatus.issued,
          dateTime: DateTime.now(),
          cardTitle: card.front.title,
          organization: issuanceResponse.organization,
          dataAttributes: card.attributes,
        ),
      );
    }
  }
}