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
    await walletRepository.unlockWallet('000000');

    // Add cards + history
    const issuanceIds = ['PID_1'];
    for (String issuanceId in issuanceIds) {
      final IssuanceResponse issuanceResponse = await issuanceResponseRepository.read(issuanceId);
      // Add card
      for (final card in issuanceResponse.cards) {
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
}
