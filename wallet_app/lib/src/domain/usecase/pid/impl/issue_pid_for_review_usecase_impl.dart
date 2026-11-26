import '../../../../data/repository/pid/pid_repository.dart';
import '../../../model/attribute/attribute.dart';
import '../../../model/result/result.dart';
import '../issue_pid_for_review_usecase.dart';

class IssuePidForReviewUseCaseImpl extends IssuePidForReviewUseCase {
  final PidRepository _pidRepository;

  IssuePidForReviewUseCaseImpl(this._pidRepository);

  @override
  Future<Result<List<Attribute>>> invoke(String pin) => tryCatch(
    () => _pidRepository.issuePidForReview(pin),
    'Failed to issue pid for review',
  );
}
