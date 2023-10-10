import '../../model/navigation/navigation_request.dart';

export '../../model/navigation/navigation_request.dart';

abstract class PerformPreNavigationActionsUseCase {
  /// Performs all the specified pre navigation actions
  Future<void> invoke(List<PreNavigationAction> actions);
}
