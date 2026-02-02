import 'package:permission_handler/permission_handler.dart';

import '../../model/permission/permission_check_result.dart';
import '../wallet_usecase.dart';
import 'check_permission_usecase.dart';

/// A use case for requesting a [Permission] from the user.
/// Refer to [CheckPermissionUseCase] if you only want to check the state.
abstract class RequestPermissionUseCase extends WalletUseCase {
  Future<PermissionCheckResult> invoke(Permission permission);
}
