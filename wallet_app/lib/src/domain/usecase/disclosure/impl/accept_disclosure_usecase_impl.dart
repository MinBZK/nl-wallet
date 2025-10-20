import '../../../model/result/result.dart';
import '../accept_disclosure_usecase.dart';

class AcceptDisclosureUseCaseImpl extends AcceptDisclosureUseCase {
  final DisclosureRepository _disclosureRepository;

  /// The indices of the cards the user chose to disclose.
  ///
  /// Each index matches a position in the candidate cards list from the corresponding [StartDisclosureResult].
  /// The order matters and the number of indices should match the number of candidates.
  ///
  /// Cards are passed in through the constructor so we can reuse [CheckPinUseCase]
  /// with the default [PinBloc] handling all UI and error flows.
  final List<int> selectedIndices;

  AcceptDisclosureUseCaseImpl(this._disclosureRepository, this.selectedIndices);

  @override
  Future<Result<String?>> invoke(String pin) async {
    return tryCatch(
      () async => _disclosureRepository.acceptDisclosure(pin, selectedIndices),
      'Failed to accept disclosure',
    );
  }
}
