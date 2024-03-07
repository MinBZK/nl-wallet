import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
import 'text_icon_button.dart';

const _kButtonHeight = 72.0;
const _kLandscapeButtonHeight = 56.0;

/// Button wrapper (mostly to wrap a [TextIconButton]) that is aligned at the
/// bottom of the screen and is rendered with a divider. Often used as a direct child of
/// a [SliverFillRemaining] widget.
class BottomButton extends StatelessWidget {
  final Widget button;

  const BottomButton({
    required this.button,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Align(
      alignment: Alignment.bottomCenter,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Divider(height: 1),
          ConstrainedBox(
            constraints: BoxConstraints(
              minHeight: context.isLandscape ? _kLandscapeButtonHeight : _kButtonHeight,
              minWidth: double.infinity,
            ),
            child: Theme(
              data: context.theme.copyWith(
                textButtonTheme: TextButtonThemeData(
                  style: context.theme.textButtonTheme.style?.copyWith(
                    // Remove rounded edges
                    shape: const MaterialStatePropertyAll(RoundedRectangleBorder()),
                  ),
                ),
              ),
              child: button,
            ),
          ),
        ],
      ),
    );
  }
}
