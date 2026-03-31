import 'package:fimber/fimber.dart';
import 'package:permission_handler/permission_handler.dart';

import '../../../model/permission/permission_check_result.dart';
import '../request_permission_usecase.dart';

class RequestPermissionUseCaseImpl extends RequestPermissionUseCase {
  RequestPermissionUseCaseImpl();

  @override
  Future<PermissionCheckResult> invoke(List<Permission> permissions) async {
    try {
      final Map<Permission, PermissionStatus> statuses = await permissions.request();
      final allGranted = statuses.values.every((status) => status.isGranted);
      if (allGranted) return const PermissionCheckResult(isGranted: true, isPermanentlyDenied: false);

      // Check if any denied permission is permanently denied.
      final isPermanentlyDenied = statuses.values.any((value) => value.isPermanentlyDenied);
      return PermissionCheckResult(isGranted: false, isPermanentlyDenied: isPermanentlyDenied);
    } catch (ex) {
      Fimber.e('Could not check permissions for: $permissions', ex: ex);
      return const PermissionCheckResult(isGranted: false, isPermanentlyDenied: false);
    }
  }
}
