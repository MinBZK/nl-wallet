import '../../model/navigation/navigation_request.dart';

abstract class CheckNavigationPrerequisitesUseCase {
  /// Returns true when all navigationPrerequisites are passed
  Future<bool> invoke(List<NavigationPrerequisite> prerequisites);
}
