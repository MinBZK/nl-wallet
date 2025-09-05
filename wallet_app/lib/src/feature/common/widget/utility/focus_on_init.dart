import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';

import '../../../../../environment.dart';
import '../../../../wallet_constants.dart';

class FocusOnInit extends StatefulWidget {
  final Widget? child;
  final bool requestFocus;

  const FocusOnInit({
    this.child,
    this.requestFocus = true,
    super.key,
  });

  @override
  State<FocusOnInit> createState() => _FocusOnInit();
}

class _FocusOnInit extends State<FocusOnInit> {
  @override
  void initState() {
    super.initState();
    if (!widget.requestFocus) return;
    WidgetsBinding.instance.addPostFrameCallback((_) async {
      if (!Environment.isTest) await Future.delayed(kDefaultAnimationDuration);
      if (!mounted) return;
      context.findRenderObject()?.sendSemanticsEvent(const FocusSemanticEvent());
    });
  }

  @override
  Widget build(BuildContext context) => widget.child ?? const SizedBox.shrink();
}
