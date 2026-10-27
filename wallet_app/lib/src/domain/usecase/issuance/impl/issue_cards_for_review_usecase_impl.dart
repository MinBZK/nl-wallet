import '../../../../data/repository/issuance/issuance_repository.dart';
import '../../../model/card/wallet_card.dart';
import '../../../model/result/result.dart';
import '../issue_cards_for_review_usecase.dart';

class IssueCardsForReviewUseCaseImpl extends IssueCardsForReviewUseCase {
  final IssuanceRepository _issuanceRepository;

  IssueCardsForReviewUseCaseImpl(this._issuanceRepository);

  @override
  Future<Result<List<WalletCard>>> invoke(String pin) => tryCatch(
    () => _issuanceRepository.issueCardsForReview(pin),
    'Failed to issue cards for review',
  );
}
