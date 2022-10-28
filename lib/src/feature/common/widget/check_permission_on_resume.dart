import 'package:flutter/material.dart';
import 'package:permission_handler/permission_handler.dart';

class CheckPermissionOnResume extends StatefulWidget {
  final Widget child;
  final VoidCallback onPermissionGranted;
  final Permission permission;

  const CheckPermissionOnResume({
    required this.child,
    required this.onPermissionGranted,
    required this.permission,
    Key? key,
  }) : super(key: key);

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

  void _checkPermission() async {
    if (await widget.permission.isGranted) {
      widget.onPermissionGranted();
    }
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }
}
