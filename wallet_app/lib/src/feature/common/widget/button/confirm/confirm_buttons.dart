import 'package:flutter/material.dart';
import 'package:provider/single_child_widget.dart';

import '../../../../../util/extension/build_context_extension.dart';
import 'horizontal_confirm_buttons.dart';
import 'vertical_confirm_buttons.dart';

const _kPadding = EdgeInsets.symmetric(horizontal: 16, vertical: 24);
const _kLandscapePadding = EdgeInsets.symmetric(horizontal: 16, vertical: 12);

/// Widget that renders two buttons either side by side, or stacked vertically,
/// depending on the content and the available screen size. Usually used on the
/// bottom of the screen.
/// NOTE: This widget assumes it takes up the full width of the screen, so it should
/// not be wrapped in any padding/margin providing widget.
class ConfirmButtons extends StatelessWidget {
  static const kButtonSpacing = 12.0;

  final FitsWidthWidget primaryButton;
  final FitsWidthWidget secondaryButton;

  final bool forceVertical;

  /// Flips the position of the primary and secondary buttons in the
  /// vertical layout.
  final bool flipVertical;

  /// Whether the [secondaryButton] should be shown,
  /// changing this will trigger an implicit animation.
  final bool hideSecondaryButton;

  const ConfirmButtons({
    required this.primaryButton,
    required this.secondaryButton,
    this.forceVertical = false,
    this.flipVertical = false,
    this.hideSecondaryButton = false,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final availableWidthPerButton = _calculateAvailableWidth(context);
    final bool buttonsFitHorizontally = primaryButton.fitsWidth(context, availableWidthPerButton) &&
        secondaryButton.fitsWidth(context, availableWidthPerButton);

    Widget buttons;
    if (!buttonsFitHorizontally || forceVertical) {
      buttons = VerticalConfirmButtons(
        primaryButton: primaryButton,
        secondaryButton: secondaryButton,
        hideSecondaryButton: hideSecondaryButton,
        flipVertical: flipVertical,
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
        padding: context.isLandscape ? _kLandscapePadding : _kPadding,
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
    final padding = context.isLandscape ? _kLandscapePadding : _kPadding;
    final double singleButtonWidth = (screenWidth - padding.horizontal - kButtonSpacing) / 2;
    return singleButtonWidth;
  }
}

/// Children of [ConfirmButtons] should implement this interface, so that [ConfirmButtons] can figure
/// out whether it should use the horizontal or vertical layout.
///
/// When you don't care about the horizontal layout you can wrap a child in [NeverFitsWidthWidget]
/// to simply render a widget in the vertical layout.
abstract class FitsWidthWidget implements Widget {
  /// Let the caller know if this widget comfortably fits in [availableWidth].
  /// When returning false the caller might decide to render this widget in a wider
  /// container.
  bool fitsWidth(BuildContext context, double availableWidth);
}

/// A basic implementation of [FitsWidthWidget] that reports it never fits the provided width.
class NeverFitsWidthWidget extends SingleChildStatelessWidget implements FitsWidthWidget {
  const NeverFitsWidthWidget({
    required super.child,
    super.key,
  });

  @override
  bool fitsWidth(BuildContext context, double availableWidth) => false;

  @override
  Widget buildWithChild(BuildContext context, Widget? child) => child!;
}
