export '../../../data/repository/disclosure/disclosure_repository.dart';

abstract class CancelDisclosureUseCase {
  /// Cancels the ongoing disclosure session and returns the returnUrl to redirect the user (when available).
  Future<String?> invoke();
}
