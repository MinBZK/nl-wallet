import 'package:permission_handler/permission_handler.dart';

import '../../model/permission/permission_check_result.dart';
import '../wallet_usecase.dart';
import 'request_permission_usecase.dart';

/// A use case for checking the state of one or more [Permission]s.
/// Refer to [RequestPermissionUseCase] if you want to request a permission.
abstract class CheckPermissionUseCase extends WalletUseCase {
  Future<PermissionCheckResult> invoke(List<Permission> permissions);
}
