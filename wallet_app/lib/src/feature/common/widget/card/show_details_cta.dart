import 'package:flutter/material.dart';

import '../../../../theme/base_wallet_theme.dart';
import '../../../../theme/light_wallet_theme.dart';
import '../../../../util/extension/build_context_extension.dart';

const _kButtonHeight = 24.0;
const _kFallBackIconSize = 16.0;

class ShowDetailsCta extends StatelessWidget {
  final Text text;
  final Color? textColor;
  final VoidCallback? onPressed;

  const ShowDetailsCta({
    required this.text,
    this.textColor,
    this.onPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return TextButton.icon(
      icon: Icon(
        Icons.arrow_forward,
        size: context.textScaler.scale(context.theme.iconTheme.size ?? _kFallBackIconSize),
      ),
      iconAlignment: IconAlignment.end,
      style: _resolveButtonStyle(context),
      onPressed: onPressed,
      label: text,
    );
  }

  ButtonStyle _resolveButtonStyle(BuildContext context) {
    return context.theme.textButtonTheme.style!.copyWith(
      backgroundColor: const WidgetStatePropertyAll(
        Colors.transparent,
      ),
      foregroundColor: WidgetStatePropertyAll(
        textColor,
      ),
      iconColor: WidgetStatePropertyAll(
        textColor,
      ),
      minimumSize: const WidgetStatePropertyAll(
        Size(0, _kButtonHeight),
      ),
      padding: const WidgetStatePropertyAll(
        EdgeInsets.zero,
      ),
      tapTargetSize: MaterialTapTargetSize.shrinkWrap,
      shape: const WidgetStatePropertyAll(
        RoundedRectangleBorder(
          borderRadius: BorderRadius.zero,
        ),
      ),
      side: WidgetStateProperty.resolveWith(
        (states) {
          final fallBackColor = context.theme.textTheme.bodyLarge?.color ?? LightWalletTheme.textColor;
          return states.isFocused ? BorderSide(color: textColor ?? fallBackColor) : null;
        },
      ),
    );
  }
}
