import 'package:wallet_mock/mock.dart';

import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../model/card/wallet_card.dart';
import '../../../model/result/result.dart';
import '../get_pid_cards_usecase.dart';

final kMockPidAttestationTypes = [MockAttestationTypes.pid, MockAttestationTypes.address];

class GetPidCardsUseCaseImpl extends GetPidCardsUseCase {
  final WalletCardRepository walletCardRepository;

  GetPidCardsUseCaseImpl(this.walletCardRepository);

  @override
  Future<Result<List<WalletCard>>> invoke() async {
    return tryCatch(
      () async {
        final cards = await walletCardRepository.readAll();
        // TODO(Rob): The attestation types should eventually be provided by the core.
        final pidCards = cards.where((card) => kMockPidAttestationTypes.contains(card.attestationType));
        return pidCards.toList();
      },
      'Failed to get pid cards',
    );
  }
}
