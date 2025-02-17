import 'package:permission_handler/permission_handler.dart';

import '../wallet_usecase.dart';

abstract class CheckHasPermissionUseCase extends WalletUseCase {
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
