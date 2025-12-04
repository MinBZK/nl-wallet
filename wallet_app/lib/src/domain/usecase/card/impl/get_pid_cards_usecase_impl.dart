import 'package:wallet_mock/mock.dart';

import '../../../../data/repository/card/wallet_card_repository.dart';
import '../../../../data/repository/configuration/configuration_repository.dart';
import '../../../model/card/wallet_card.dart';
import '../../../model/result/result.dart';
import '../get_pid_cards_usecase.dart';

final kMockPidAttestationTypes = [MockAttestationTypes.pid, MockAttestationTypes.address];

class GetPidCardsUseCaseImpl extends GetPidCardsUseCase {
  final WalletCardRepository _walletCardRepository;
  final ConfigurationRepository _configurationRepository;

  GetPidCardsUseCaseImpl(this._walletCardRepository, this._configurationRepository);

  @override
  Future<Result<List<WalletCard>>> invoke() async {
    return tryCatch(
      () async {
        final cards = await _walletCardRepository.readAll();
        final config = await _configurationRepository.appConfiguration.first;
        final pidCards = cards.where((card) => config.pidAttestationTypes.contains(card.attestationType));
        return pidCards.toList();
      },
      'Failed to get pid cards',
    );
  }
}
