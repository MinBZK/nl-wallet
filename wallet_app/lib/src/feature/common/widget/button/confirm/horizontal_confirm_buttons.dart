import 'package:flutter/cupertino.dart';

import '../../../../../wallet_constants.dart';
import 'confirm_buttons.dart';

class HorizontalConfirmButtons extends StatefulWidget {
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
    controller = AnimationController(
      vsync: this,
      duration: kDefaultAnimationDuration,
      value: widget.hideSecondaryButton ? 1 : 0,
    );
    animation = CurvedAnimation(
      parent: controller,
      curve: Curves.easeInOutCubic,
    );
    super.initState();
  }

  @override
  void dispose() {
    controller.dispose();
    super.dispose();
  }

  @override
  void didUpdateWidget(covariant HorizontalConfirmButtons oldWidget) {
    controller.animateTo(widget.hideSecondaryButton ? 1 : 0);
    super.didUpdateWidget(oldWidget);
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: controller,
      builder: (context, child) {
        const defaultButtonSizeAsFraction = 0.5;
        final primarySizeTween = Tween(begin: defaultButtonSizeAsFraction, end: 1.0);
        final secondaryButtonXAlignmentTween = Tween(begin: -1.0, end: -3.0);
        final centralPaddingTween = Tween(begin: ConfirmButtons.kButtonSpacing / 2.0, end: 0.0);
        return Stack(
          clipBehavior: Clip.none,
          children: [
            Align(
              alignment: Alignment(secondaryButtonXAlignmentTween.evaluate(animation), 0.0),
              child: FractionallySizedBox(
                widthFactor: defaultButtonSizeAsFraction,
                child: Padding(
                  padding: EdgeInsets.only(right: centralPaddingTween.evaluate(animation)),
                  child: widget.secondaryButton,
                ),
              ),
            ),
            Align(
              alignment: Alignment.centerRight,
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
