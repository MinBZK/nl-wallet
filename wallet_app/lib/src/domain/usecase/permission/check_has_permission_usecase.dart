import 'package:permission_handler/permission_handler.dart';

abstract class CheckHasPermissionUseCase {
  Future<PermissionCheckResult> invoke(Permission permission);
}

class PermissionCheckResult {
  final bool isGranted;
  final bool isPermanentlyDenied;

  PermissionCheckResult({required this.isGranted, required this.isPermanentlyDenied})
      : assert(
          (isGranted && !isPermanentlyDenied) || !isGranted,
          'Permission can not be both granted and permanently denied',
        );
}
