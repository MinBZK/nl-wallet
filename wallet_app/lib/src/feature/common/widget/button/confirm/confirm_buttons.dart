import 'package:flutter/material.dart';

import '../../../../../util/extension/build_context_extension.dart';
import 'confirm_button.dart';
import 'horizontal_confirm_buttons.dart';
import 'vertical_confirm_buttons.dart';

const _kHorizontalPadding = 16.0;
const _kVerticalPadding = 24.0;
const _kVerticalLandscapePadding = 12.0;

class ConfirmButtons extends StatelessWidget {
  static const kButtonSpacing = 12.0;

  final ConfirmButton primaryButton;
  final ConfirmButton secondaryButton;

  /// Other config
  final bool forceVertical;

  /// Whether the [secondaryButton] should be shown,
  /// changing this will trigger an implicit animation.
  final bool hideSecondaryButton;

  const ConfirmButtons({
    required this.primaryButton,
    required this.secondaryButton,
    this.forceVertical = false,
    this.hideSecondaryButton = false,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final availableWidthPerButton = _calculateAvailableWidth(context);
    final bool buttonsFitOnSingleLine = primaryButton.fitsOnSingleLine(context, availableWidthPerButton) &&
        secondaryButton.fitsOnSingleLine(context, availableWidthPerButton);

    Widget buttons;
    if (!buttonsFitOnSingleLine || forceVertical) {
      buttons = VerticalConfirmButtons(
        primaryButton: primaryButton,
        secondaryButton: secondaryButton,
        hideSecondaryButton: hideSecondaryButton,
      );
    } else {
      buttons = HorizontalConfirmButtons(
        primaryButton: primaryButton,
        secondaryButton: secondaryButton,
        hideSecondaryButton: hideSecondaryButton,
      );
    }

    return SafeArea(
      top: false,
      child: Padding(
        padding: EdgeInsets.symmetric(
          horizontal: _kHorizontalPadding,
          vertical: context.isLandscape ? _kVerticalLandscapePadding : _kVerticalPadding,
        ),
        child: buttons,
      ),
    );
  }

  /// Calculate the available width for a confirm button
  /// (so that if fills half of the screen and takes spacing/padding into account)
  ///
  /// Note that this assumes the [ConfirmButtons] widget takes up the full width of the screen
  double _calculateAvailableWidth(BuildContext context) {
    final double screenWidth = context.mediaQuery.size.width.roundToDouble();
    final double singleButtonWidth = (screenWidth - (_kHorizontalPadding * 2) - kButtonSpacing) / 2;
    return singleButtonWidth;
  }
}
