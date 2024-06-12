import 'package:collection/collection.dart';
import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';

import '../../../../../wallet_constants.dart';
import 'confirm_buttons.dart';

class VerticalConfirmButtons extends StatefulWidget {
  final Widget primaryButton;
  final Widget secondaryButton;
  final bool hideSecondaryButton;
  final bool flipVertical;

  const VerticalConfirmButtons({
    required this.primaryButton,
    required this.secondaryButton,
    this.hideSecondaryButton = false,
    this.flipVertical = false,
    super.key,
  });

  @override
  State<VerticalConfirmButtons> createState() => _VerticalConfirmButtonsState();
}

class _VerticalConfirmButtonsState extends State<VerticalConfirmButtons> with SingleTickerProviderStateMixin {
  late AnimationController controller;
  late Animation<double> animation;

  @override
  void initState() {
    super.initState();
    controller = AnimationController(
      vsync: this,
      duration: kDefaultAnimationDuration,
      value: widget.hideSecondaryButton ? 1 : 0,
    );
    animation = CurvedAnimation(
      parent: controller,
      curve: Curves.easeInCubic,
    );
  }

  @override
  void dispose() {
    controller.dispose();
    super.dispose();
  }

  @override
  void didUpdateWidget(covariant VerticalConfirmButtons oldWidget) {
    super.didUpdateWidget(oldWidget);
    controller.animateTo(widget.hideSecondaryButton ? 1 : 0);
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: controller,
      builder: (context, child) {
        final yScaleTween = Tween<double>(begin: 1, end: 0);
        final opacityTween = TweenSequence<double>([
          TweenSequenceItem(tween: Tween<double>(begin: 1, end: 0), weight: 1),
          TweenSequenceItem(tween: Tween<double>(begin: 0, end: 0), weight: 3),
        ]);
        final columnChildren = [
          widget.primaryButton,
          SizedBox(height: ConfirmButtons.kButtonSpacing * yScaleTween.evaluate(animation)),
          Opacity(
            opacity: opacityTween.evaluate(animation),
            child: SizeTransition(
              sizeFactor: animation.drive(yScaleTween),
              child: widget.secondaryButton,
            ),
          ),
        ];
        return Column(
          mainAxisSize: MainAxisSize.min,
          children: columnChildren..reverseRange(0, widget.flipVertical ? columnChildren.length : 0),
        );
      },
    );
  }
}
