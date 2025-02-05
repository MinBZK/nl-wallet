import 'package:flutter/material.dart';

import '../../../../theme/base_wallet_theme.dart';
import '../../../../theme/dark_wallet_theme.dart';
import '../../../../theme/light_wallet_theme.dart';
import '../../../../util/extension/build_context_extension.dart';

const _kButtonHeight = 24.0;
const _kLightBrightnessTextColor = LightWalletTheme.textColor;
const _kDarkBrightnessTextColor = DarkWalletTheme.textColor;

class ShowDetailsCta extends StatelessWidget {
  final Brightness brightness;
  final Text text;
  final VoidCallback? onPressed;

  const ShowDetailsCta({
    required this.brightness,
    required this.text,
    this.onPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return TextButton.icon(
      icon: const Icon(Icons.arrow_forward_ios),
      iconAlignment: IconAlignment.end,
      style: _resolveButtonStyle(context),
      onPressed: onPressed,
      label: text,
    );
  }

  ButtonStyle _resolveButtonStyle(BuildContext context) {
    final textColor = brightness == Brightness.light ? _kLightBrightnessTextColor : _kDarkBrightnessTextColor;
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
        (states) => states.isFocused ? BorderSide(color: textColor) : null,
      ),
    );
  }
}
