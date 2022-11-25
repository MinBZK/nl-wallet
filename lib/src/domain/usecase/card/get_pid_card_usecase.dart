import '../../../data/repository/issuance/issuance_response_repository.dart';
import '../../model/issuance_response.dart';
import '../../model/wallet_card.dart';

const _kPidCardId = 'PID_1';

class GetPidCardUseCase {
  final IssuanceResponseRepository issuanceResponseRepository;

  GetPidCardUseCase(this.issuanceResponseRepository);

  Future<WalletCard> invoke() async {
    final IssuanceResponse issuanceResponse = await issuanceResponseRepository.read(_kPidCardId);
    return issuanceResponse.cards.first;
  }
}
