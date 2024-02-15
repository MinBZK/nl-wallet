import 'package:flutter/material.dart';

import '../../../../wallet_constants.dart';
import 'wallet_app_bar_back_button.dart';

class AnimatedVisibilityBackButton extends StatelessWidget {
  final VoidCallback? onPressed;
  final bool visible;

  const AnimatedVisibilityBackButton({
    required this.visible,
    this.onPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return AnimatedOpacity(
      opacity: visible ? 1 : 0,
      duration: kDefaultAnimationDuration,
      child: IgnorePointer(
        ignoring: !visible,
        child: WalletAppBarBackButton(
          onPressed: onPressed,
        ),
      ),
    );
  }
}
