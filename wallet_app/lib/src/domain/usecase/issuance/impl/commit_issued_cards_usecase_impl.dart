import '../../../../data/repository/issuance/issuance_repository.dart';
import '../../../model/result/result.dart';
import '../commit_issued_cards_usecase.dart';

class CommitIssuedCardsUseCaseImpl extends CommitIssuedCardsUseCase {
  final IssuanceRepository _issuanceRepository;

  CommitIssuedCardsUseCaseImpl(this._issuanceRepository);

  @override
  Future<Result<void>> invoke() => tryCatch(
    _issuanceRepository.commitIssuedCards,
    'Failed to commit issued cards',
  );
}
