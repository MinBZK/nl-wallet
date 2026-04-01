import 'package:fimber/fimber.dart';
import 'package:permission_handler/permission_handler.dart';

import '../../../model/permission/permission_check_result.dart';
import '../check_permission_usecase.dart';

class CheckPermissionUseCaseImpl extends CheckPermissionUseCase {
  CheckPermissionUseCaseImpl();

  @override
  Future<PermissionCheckResult> invoke(List<Permission> permissions) async {
    try {
      // Check permissions sequentially; no batch API available for permission checks.
      for (final permission in permissions) {
        final isGranted = await permission.isGranted;
        if (!isGranted) {
          final isPermanentlyDenied = await permission.isPermanentlyDenied;
          return PermissionCheckResult(isGranted: false, isPermanentlyDenied: isPermanentlyDenied);
        }
      }
      return const PermissionCheckResult(isGranted: true, isPermanentlyDenied: false);
    } catch (ex) {
      Fimber.e('Could not check permissions for: $permissions', ex: ex);
      return const PermissionCheckResult(isGranted: false, isPermanentlyDenied: false);
    }
  }
}
