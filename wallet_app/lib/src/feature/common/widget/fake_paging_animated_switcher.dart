import 'package:flutter/material.dart';

import '../../../wallet_constants.dart';

class FakePagingAnimatedSwitcher extends StatelessWidget {
  final Widget child;
  final bool animateBackwards;

  const FakePagingAnimatedSwitcher({required this.child, this.animateBackwards = false, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return AnimatedSwitcher(
      duration: kDefaultAnimationDuration,
      switchOutCurve: Curves.easeInOut,
      switchInCurve: Curves.easeInOut,
      transitionBuilder: (child, animation) {
        return DualTransitionBuilder(
          animation: animation,
          forwardBuilder: (context, animation, forwardChild) => SlideTransition(
            position: Tween<Offset>(
              begin: Offset(animateBackwards ? 0 : 1, 0),
              end: Offset(animateBackwards ? 1 : 0, 0),
            ).animate(animation),
            child: forwardChild,
          ),
          reverseBuilder: (context, animation, reverseChild) => SlideTransition(
            position: Tween<Offset>(
              begin: Offset(animateBackwards ? -1 : 0, 0),
              end: Offset(animateBackwards ? 0 : -1, 0),
            ).animate(animation),
            child: reverseChild,
          ),
          child: child,
        );
      },
      child: child,
    );
  }
}
