import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../card/get_pid_issuance_response_usecase.dart';
import '../../card/wallet_add_issued_cards_usecase.dart';
import '../accept_offered_pid_usecase.dart';

/// This usecase checks the pin, and when the provided pin matches it inserts
/// the pid (personal data & address) cards. This aligns with what happens in the
/// wallet_core implementation, and makes sure we can consecutively fetch the
/// pid by simply getting all cards.
class AcceptOfferedPidUseCaseMock implements AcceptOfferedPidUseCase {
  final WalletRepository walletRepository;
  final GetPidIssuanceResponseUseCase getPidIssuanceResponseUseCase;
  final WalletAddIssuedCardsUseCase walletAddIssuedCardsUseCase;

  const AcceptOfferedPidUseCaseMock(
    this.walletRepository,
    this.getPidIssuanceResponseUseCase,
    this.walletAddIssuedCardsUseCase,
  );

  @override
  Future<CheckPinResult> invoke(String pin) async {
    final result = await walletRepository.confirmTransaction(pin);
    if (result is CheckPinResultOk) {
      final issuanceResponse = await getPidIssuanceResponseUseCase.invoke();
      await walletAddIssuedCardsUseCase.invoke(issuanceResponse.cards, issuanceResponse.organization);
    }
    return result;
  }
}
