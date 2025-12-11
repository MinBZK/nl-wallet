import 'package:fimber/fimber.dart';
import 'package:permission_handler/permission_handler.dart';

import '../../../model/permission/permission_check_result.dart';
import '../check_permission_usecase.dart';

class CheckPermissionUseCaseImpl extends CheckPermissionUseCase {
  CheckPermissionUseCaseImpl();

  @override
  Future<PermissionCheckResult> invoke(Permission permission) async {
    try {
      final isGranted = await permission.isGranted;
      if (isGranted) return PermissionCheckResult(isGranted: isGranted, isPermanentlyDenied: false);
      final isPermanentlyDenied = await permission.isPermanentlyDenied;
      return PermissionCheckResult(isGranted: isGranted, isPermanentlyDenied: isPermanentlyDenied);
    } catch (ex) {
      Fimber.e('Could not check permission for: $permission', ex: ex);
      // Return a sane default that would cause us to try again.
      return const PermissionCheckResult(isGranted: false, isPermanentlyDenied: false);
    }
  }
}
