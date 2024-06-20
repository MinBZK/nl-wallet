import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_constants.dart';

class FakePagingAnimatedSwitcher extends StatelessWidget {
  /// The [child] contained by the FakePagingAnimatedSwitcher.
  final Widget child;

  /// When set to false, the new [child] is shown directly, without any animation.
  final bool animate;

  /// When set to true, the 'slide' transition is reversed, i.e. the new [child] is animated in from the left.
  final bool animateBackwards;

  const FakePagingAnimatedSwitcher({
    required this.child,
    this.animate = true,
    this.animateBackwards = false,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    // Used to disable animations when the screenReader is on
    final screenReaderEnabled = context.isScreenReaderEnabled;
    return AnimatedSwitcher(
      reverseDuration: screenReaderEnabled ? Duration.zero : kDefaultAnimationDuration,
      duration: animate && !screenReaderEnabled ? kDefaultAnimationDuration : Duration.zero,
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

      /// Wrapping the child with [PrimaryScrollController] to fix (potential) crashes that are caused by
      /// concurrent use of the same [ScrollController].
      /// Without this explicit [PrimaryScrollController] any child using the [PrimaryScrollController]
      /// relies on the same [ScrollController], which can be problematic since the [FakePagingAnimatedSwitcher]
      /// (by design) can render multiple [child]s at the same time, causing a conflict.
      /// Key is set to differentiate between different [child]s and is based on a combination of the [child]'s
      /// runtime type and it's (custom) key.
      child: PrimaryScrollController(
        controller: ScrollController(),
        key: ValueKey('${child.runtimeType}-${child.key}'),
        child: child,
      ),
    );
  }
}
