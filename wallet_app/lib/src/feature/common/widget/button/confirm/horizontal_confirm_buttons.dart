import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';

import '../../../../../wallet_constants.dart';
import 'confirm_buttons.dart';

class HorizontalConfirmButtons extends StatefulWidget {
  @visibleForTesting
  static const secondaryButtonAlignmentKey = Key('secondaryButtonAlignment');
  @visibleForTesting
  static const kHiddenXAlignment = -3.2;

  final Widget primaryButton;
  final Widget secondaryButton;
  final bool hideSecondaryButton;

  const HorizontalConfirmButtons({
    required this.primaryButton,
    required this.secondaryButton,
    this.hideSecondaryButton = false,
    super.key,
  });

  @override
  State<HorizontalConfirmButtons> createState() => _HorizontalConfirmButtonsState();
}

class _HorizontalConfirmButtonsState extends State<HorizontalConfirmButtons> with SingleTickerProviderStateMixin {
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
      curve: Curves.easeInOutCubic,
    );
  }

  @override
  void dispose() {
    controller.dispose();
    super.dispose();
  }

  @override
  void didUpdateWidget(covariant HorizontalConfirmButtons oldWidget) {
    super.didUpdateWidget(oldWidget);
    controller.animateTo(widget.hideSecondaryButton ? 1 : 0);
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: controller,
      builder: (context, child) {
        const defaultButtonSizeAsFraction = 0.5;
        final primarySizeTween = Tween<double>(begin: defaultButtonSizeAsFraction, end: 1);
        final secondaryButtonXAlignmentTween =
            Tween<double>(begin: -1, end: HorizontalConfirmButtons.kHiddenXAlignment);
        final centralPaddingTween = Tween<double>(begin: ConfirmButtons.kButtonSpacing / 2.0, end: 0);
        return Stack(
          clipBehavior: Clip.none,
          children: [
            Align(
              key: HorizontalConfirmButtons.secondaryButtonAlignmentKey,
              alignment: Alignment(secondaryButtonXAlignmentTween.evaluate(animation), 1),
              child: FractionallySizedBox(
                widthFactor: defaultButtonSizeAsFraction,
                child: Padding(
                  padding: EdgeInsets.only(right: centralPaddingTween.evaluate(animation)),
                  child: ExcludeFocus(
                    excluding: animation.value == 1,
                    child: ExcludeSemantics(
                      excluding: animation.value == 1,
                      child: widget.secondaryButton,
                    ),
                  ),
                ),
              ),
            ),
            Align(
              alignment: Alignment.bottomRight,
              child: FractionallySizedBox(
                widthFactor: primarySizeTween.evaluate(animation),
                child: Padding(
                  padding: EdgeInsets.only(left: centralPaddingTween.evaluate(animation)),
                  child: widget.primaryButton,
                ),
              ),
            ),
          ],
        );
      },
    );
  }
}
