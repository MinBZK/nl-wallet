import 'package:flutter/material.dart';

import '../../../../theme/base_wallet_theme.dart';
import '../../../../util/extension/build_context_extension.dart';
import 'video_overlay.dart';

class VideoControlTextButton extends StatelessWidget {
  final IconData icon;
  final String label;
  final VoidCallback onPressed;

  const VideoControlTextButton({
    required this.icon,
    required this.label,
    required this.onPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final BorderSide buttonBorderSideFocused = BaseWalletTheme.buttonBorderSideFocused.copyWith(
      color: Colors.white,
    );

    return TextButton.icon(
      style: context.theme.iconButtonTheme.style?.copyWith(
        backgroundColor: WidgetStateProperty.resolveWith(
          (states) => states.isPressedOrFocused ? kVideoControlPressedOrFocusedBg : kVideoControlDefaultBg,
        ),
        foregroundColor: const WidgetStatePropertyAll(Colors.white),
        iconColor: const WidgetStatePropertyAll(Colors.white),
        padding: const WidgetStatePropertyAll(
          EdgeInsets.only(left: 12, top: 8, right: 16, bottom: 8),
        ),
        shape: WidgetStateProperty.all(
          const RoundedRectangleBorder(
            borderRadius: BorderRadius.all(Radius.circular(4)),
          ),
        ),
        side: WidgetStateProperty.resolveWith(
          (states) => states.isFocused ? buttonBorderSideFocused : null,
        ),
      ),
      icon: Container(
        alignment: Alignment.center,
        width: 30,
        height: 30,
        child: Icon(icon),
      ),
      label: Text(label),
      onPressed: onPressed,
    );
  }
}
