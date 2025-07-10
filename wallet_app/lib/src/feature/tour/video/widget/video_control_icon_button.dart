import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';

import '../../../../theme/base_wallet_theme.dart';
import '../../../../util/extension/build_context_extension.dart';
import 'video_overlay.dart';

/// The default size for the [VideoControlIconButton].
const double kDefaultVideoControlIconButtonSize = 56;

/// The default shape for the [VideoControlIconButton].
const _kDefaultShape = RoundedRectangleBorder(borderRadius: BorderRadius.all(Radius.circular(4)));

/// A widget that displays an icon button with a specific style for video controls.
class VideoControlIconButton extends StatelessWidget {
  /// The icon to be displayed on the button, typically an [Icon] widget.
  final Widget icon;

  /// The callback that is called when the button is tapped.
  final VoidCallback onPressed;

  /// The shape of the button.
  /// Defaults to a [RoundedRectangleBorder] with a circular radius of 4. See [_kDefaultShape].
  final OutlinedBorder shape;

  /// The attributed tooltip text to be used for semantics.
  final AttributedString? attributedTooltip;

  /// Creates a [VideoControlIconButton].
  const VideoControlIconButton({
    required this.icon,
    required this.onPressed,
    this.shape = _kDefaultShape,
    this.attributedTooltip,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final BorderSide buttonBorderSideFocused = BaseWalletTheme.buttonBorderSideFocused.copyWith(color: Colors.white);
    final iconButton = SizedBox(
      width: kDefaultVideoControlIconButtonSize,
      height: kDefaultVideoControlIconButtonSize,
      child: IconButton(
        style: context.theme.iconButtonTheme.style?.copyWith(
          iconColor: const WidgetStatePropertyAll(Colors.white),
          shape: WidgetStatePropertyAll(shape),
          backgroundColor: WidgetStateProperty.resolveWith(
            (states) => states.isPressedOrFocused ? kVideoControlPressedOrFocusedBg : kVideoControlDefaultBg,
          ),
          side: WidgetStateProperty.resolveWith(
            (states) => states.isFocused ? buttonBorderSideFocused : null,
          ),
        ),
        onPressed: onPressed,
        icon: icon,
      ),
    );

    return Semantics(
      attributedLabel: attributedTooltip,
      excludeSemantics: true,
      button: true,
      container: true,
      onTap: onPressed,
      child: iconButton,
    );
  }
}
