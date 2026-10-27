import '../../../../data/repository/issuance/issuance_repository.dart';
import '../../../model/result/result.dart';
import '../accept_issued_cards_usecase.dart';

class AcceptIssuedCardsUseCaseImpl extends AcceptIssuedCardsUseCase {
  final IssuanceRepository _issuanceRepository;

  AcceptIssuedCardsUseCaseImpl(this._issuanceRepository);

  @override
  Future<Result<void>> invoke() => tryCatch(
    _issuanceRepository.acceptIssuedCards,
    'Failed to accept issued cards',
  );
}
