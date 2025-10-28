import '../../../../data/repository/issuance/issuance_repository.dart';
import '../../../model/card/wallet_card.dart';
import '../disclose_for_issuance_usecase.dart';

class DiscloseForIssuanceUseCaseImpl extends DiscloseForIssuanceUseCase {
  final IssuanceRepository _issuanceRepository;

  /// The indices of the cards the user chose to disclose.
  ///
  /// Each index matches a position in the candidate cards list from the corresponding [StartIssuanceResult].
  /// The order matters and the number of indices should match the number of candidates.
  ///
  /// Cards are passed in through the constructor so we can reuse [CheckPinUseCase]
  /// with the default [PinBloc] handling all UI and error flows.
  final List<int> selectedIndices;

  DiscloseForIssuanceUseCaseImpl(this._issuanceRepository, this.selectedIndices);

  @override
  Future<Result<List<WalletCard>>> invoke(String pin) => tryCatch(
    () => _issuanceRepository.discloseForIssuance(pin, selectedIndices),
    'Failed to disclose for issuance',
  );
}
