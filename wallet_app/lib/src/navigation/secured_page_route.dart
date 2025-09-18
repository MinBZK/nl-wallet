import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../data/repository/wallet/wallet_repository.dart';
import '../feature/pin/pin_overlay.dart';

const _kSlideTransitionDuration = Duration(milliseconds: 500);

/// Static reference to override the duration of a transition once,
/// set it through [SecuredPageRoute.overrideDurationOfNextTransition].
Duration? _nextTransitionDuration;

class SecuredPageRoute<T> extends MaterialPageRoute<T> {
  final SecuredPageTransition transition;

  @override
  Duration get transitionDuration {
    final transitionDuration = _nextTransitionDuration;
    if (transitionDuration != null) {
      _nextTransitionDuration = null;
      return transitionDuration;
    }
    switch (transition) {
      case SecuredPageTransition.platform:
        return super.transitionDuration;
      case SecuredPageTransition.slideInFromBottom:
        return _kSlideTransitionDuration;
    }
  }

  SecuredPageRoute({
    required WidgetBuilder builder,
    this.transition = SecuredPageTransition.platform,
    super.settings,
  }) : super(
         builder: (context) => PinOverlay(
           isLockedStream: context.read<WalletRepository>().isLockedStream,
           child: builder(context),
         ),
       );

  @override
  Widget buildTransitions(
    BuildContext context,
    Animation<double> animation,
    Animation<double> secondaryAnimation,
    Widget child,
  ) {
    switch (transition) {
      case SecuredPageTransition.platform:
        return super.buildTransitions(context, animation, secondaryAnimation, child);
      case SecuredPageTransition.slideInFromBottom:
        return _buildSlideInFromBottomTransitions(animation, child);
    }
  }

  Widget _buildSlideInFromBottomTransitions(Animation<double> animation, Widget child) {
    final curveTween = CurveTween(curve: Curves.easeInOutCubic);
    final offsetTween = Tween(begin: const Offset(0, 1), end: Offset.zero);
    final offsetAnimation = animation.drive(curveTween).drive(offsetTween);
    return SlideTransition(position: offsetAnimation, child: child);
  }

  /// Set a custom duration for the next transition to a [SecuredPageRoute], passing null instantly resets it to the defaults.
  static void overrideDurationOfNextTransition(Duration? duration) => _nextTransitionDuration = duration;
}

enum SecuredPageTransition { platform, slideInFromBottom }
