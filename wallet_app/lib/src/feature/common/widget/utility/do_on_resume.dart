import 'package:flutter/widgets.dart';

class DoOnResume extends StatefulWidget {
  const DoOnResume({
    required this.child,
    required this.onResume,
    super.key,
  });

  final Widget child;
  final VoidCallback onResume;

  @override
  State<DoOnResume> createState() => _DoOnResumeState();
}

class _DoOnResumeState extends State<DoOnResume> with WidgetsBindingObserver {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      widget.onResume();
    }
  }

  @override
  Widget build(BuildContext context) {
    return widget.child;
  }
}
