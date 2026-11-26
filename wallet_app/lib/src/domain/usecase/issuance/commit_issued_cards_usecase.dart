import '../../model/result/result.dart';
import '../wallet_usecase.dart';

/// Adds the cards that were issued by [IssueCardsForReviewUseCase] to the wallet, does not require the pin.
abstract class CommitIssuedCardsUseCase extends WalletUseCase {
  Future<Result<void>> invoke();
}
