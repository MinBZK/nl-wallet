import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:permission_handler/permission_handler.dart';

import '../../../../domain/usecase/permission/check_permission_usecase.dart';
import 'do_on_resume.dart';

class CheckPermissionsOnResume extends StatelessWidget {
  final Widget child;
  final VoidCallback? onPermissionGranted;
  final VoidCallback? onPermissionDenied;
  final List<Permission> permissions;

  @visibleForTesting
  final CheckPermissionUseCase? checkPermissionUseCase;

  const CheckPermissionsOnResume({
    required this.child,
    this.onPermissionGranted,
    this.onPermissionDenied,
    required this.permissions,
    this.checkPermissionUseCase,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return DoOnResume(
      onResume: () => _checkPermission(context),
      child: child,
    );
  }

  Future<void> _checkPermission(BuildContext context) async {
    final usecase = checkPermissionUseCase ?? context.read<CheckPermissionUseCase>();
    final result = await usecase.invoke(permissions);

    if (!context.mounted) return;

    if (result.isGranted) {
      onPermissionGranted?.call();
    } else {
      onPermissionDenied?.call();
    }
  }
}
