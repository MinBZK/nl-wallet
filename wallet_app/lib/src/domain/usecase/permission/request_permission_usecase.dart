import 'package:permission_handler/permission_handler.dart';

import '../../model/permission/permission_check_result.dart';
import '../wallet_usecase.dart';
import 'check_permission_usecase.dart';

/// A use case for requesting one or more [Permission]s from the user.
/// Refer to [CheckPermissionUseCase] if you only want to check the state.
///
/// When multiple permissions are requested, the OS groups them by permission
/// group (e.g. NEARBY_DEVICES) and shows one dialog per group. Permissions
/// within the same group are combined into a single dialog.
abstract class RequestPermissionUseCase extends WalletUseCase {
  Future<PermissionCheckResult> invoke(List<Permission> permissions);
}
