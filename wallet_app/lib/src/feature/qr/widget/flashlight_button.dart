import 'package:flutter/material.dart';

import '../../../theme/base_wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../common/widget/button/button_content.dart';

class FlashlightButton extends StatelessWidget {
  final VoidCallback onPressed;
  final bool isOn;

  const FlashlightButton({
    required this.isOn,
    required this.onPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      onTap: onPressed,
      excludeSemantics: true,
      onTapHint: context.l10n.flashlightButtonTapHint,
      attributedLabel: (isOn ? context.l10n.generalOn : context.l10n.generalOff).toAttributedString(context),
      child: OutlinedButton(
        onPressed: onPressed,
        style: context.theme.outlinedButtonTheme.style?.copyWith(
          backgroundColor: WidgetStateProperty.resolveWith((states) {
            if (states.isPressedOrFocused) return context.colorScheme.surface;
            return context.theme.outlinedButtonTheme.style!.backgroundColor!.resolve(states);
          }),
          shape: WidgetStatePropertyAll(RoundedRectangleBorder(borderRadius: BorderRadius.circular(50))),
        ),
        child: ButtonContent(
          text: Text(isOn ? context.l10n.qrScreenDisableTorchCta : context.l10n.qrScreenEnableTorchCta),
          icon: isOn ? Icon(Icons.flashlight_on_outlined) : Icon(Icons.flashlight_off_outlined),
        ),
      ),
    );
  }
}
