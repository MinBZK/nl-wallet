import '../../../../data/repository/issuance/issuance_repository.dart';
import '../../../model/card/wallet_card.dart';
import '../../../model/result/result.dart';
import '../accept_issuance_usecase.dart';

class AcceptIssuanceUseCaseImpl extends AcceptIssuanceUseCase {
  final IssuanceRepository _issuanceRepository;

  /// The cards that the user would like to add to her wallet. We provide the
  /// cards through the constructor, so that we can keep a consistent implementation
  /// of [CheckPinUseCase] going and thus rely on the default [PinBloc] setup to handle
  /// all ui and error flows.
  final List<WalletCard> cards;

  AcceptIssuanceUseCaseImpl(this._issuanceRepository, {required this.cards});

  @override
  Future<Result<void>> invoke(String pin) async {
    return tryCatch(
      () async => _issuanceRepository.acceptIssuance(pin, cards),
      'Failed to accept issuance',
    );
  }
}
