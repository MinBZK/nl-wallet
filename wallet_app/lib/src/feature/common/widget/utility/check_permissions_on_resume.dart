import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:permission_handler/permission_handler.dart';

import '../../../../domain/usecase/permission/check_permission_usecase.dart';

class CheckPermissionsOnResume extends StatefulWidget {
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
  State<CheckPermissionsOnResume> createState() => _CheckPermissionsOnResumeState();
}

class _CheckPermissionsOnResumeState extends State<CheckPermissionsOnResume> with WidgetsBindingObserver {
  @override
  Widget build(BuildContext context) {
    return widget.child;
  }

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) _checkPermission();
  }

  Future<void> _checkPermission() async {
    final usecase = widget.checkPermissionUseCase ?? context.read<CheckPermissionUseCase>();
    final result = await usecase.invoke(widget.permissions);
    if (!mounted) return;
    if (result.isGranted) {
      widget.onPermissionGranted?.call();
    } else {
      widget.onPermissionDenied?.call();
    }
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }
}
