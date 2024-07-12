import 'package:flutter/material.dart';
import 'package:permission_handler/permission_handler.dart';

typedef PermissionChecker = Future<bool> Function(Permission);

class CheckPermissionOnResume extends StatefulWidget {
  final Widget child;
  final VoidCallback onPermissionGranted;
  final Permission permission;

  /// Provide an alternative for the default [permission].isGranted check.
  /// Useful for testing, since isGranted is an extension method and can't be mocked.
  @visibleForTesting
  final PermissionChecker? checkPermission;

  const CheckPermissionOnResume({
    required this.child,
    required this.onPermissionGranted,
    required this.permission,
    this.checkPermission,
    super.key,
  });

  @override
  State<CheckPermissionOnResume> createState() => _CheckPermissionOnResumeState();
}

class _CheckPermissionOnResumeState extends State<CheckPermissionOnResume> with WidgetsBindingObserver {
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
    if (await (widget.checkPermission?.call(widget.permission) ?? widget.permission.isGranted)) {
      widget.onPermissionGranted();
    }
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }
}
