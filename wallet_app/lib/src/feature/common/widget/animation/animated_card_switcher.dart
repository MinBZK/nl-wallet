import 'package:flutter/material.dart';

import '../../../../wallet_constants.dart';

class AnimatedCardSwitcher extends StatelessWidget {
  final Widget child;
  final bool enableAnimation;

  const AnimatedCardSwitcher({
    required this.child,
    this.enableAnimation = true,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    if (!enableAnimation) return child;
    return AnimatedSwitcher(
      duration: kDefaultAnimationDuration,
      child: child,
      transitionBuilder: (child, animation) {
        final isForward = animation.status == AnimationStatus.forward || animation.status == AnimationStatus.completed;
        return ScaleTransition(
          scale: CurvedAnimation(parent: animation, curve: Curves.easeInOutCubic).drive(
            Tween<double>(begin: 0.8, end: 1),
          ),
          child: SlideTransition(
            position: CurvedAnimation(parent: animation, curve: Curves.easeInOutCubic).drive(
              Tween<Offset>(begin: Offset(isForward ? -1.2 : 1.2, 0), end: Offset.zero),
            ),
            child: child,
          ),
        );
      },
    );
  }
}
