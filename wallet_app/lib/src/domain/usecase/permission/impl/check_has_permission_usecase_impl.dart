import 'package:fimber/fimber.dart';
import 'package:permission_handler/permission_handler.dart';

import '../check_has_permission_usecase.dart';

class CheckHasPermissionUseCaseImpl extends CheckHasPermissionUseCase {
  CheckHasPermissionUseCaseImpl();

  @override
  Future<PermissionCheckResult> invoke(Permission permission) async {
    try {
      // Request the permission and check the status.
      final PermissionStatus status = await permission.request();
      final isGranted = status.isGranted;
      if (isGranted) return PermissionCheckResult(isGranted: isGranted, isPermanentlyDenied: false);
      final isPermanentlyDenied = await permission.isPermanentlyDenied;
      return PermissionCheckResult(isGranted: isGranted, isPermanentlyDenied: isPermanentlyDenied);
    } catch (ex) {
      Fimber.e('Could not check permission for: $permission', ex: ex);
      // Return a sane default that would cause us to try again.
      return PermissionCheckResult(isGranted: false, isPermanentlyDenied: false);
    }
  }
}
