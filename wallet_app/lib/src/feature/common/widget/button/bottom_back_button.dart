import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
import 'text_icon_button.dart';

const _kButtonHeight = 72.0;
const _kLandscapeButtonHeight = 56.0;

/// Back button that is aligned at the bottom of the screen,
/// can be rendered with divider by setting [showDivider].
/// Often used as a direct child of a [SliverFillRemaining] widget.
class BottomBackButton extends StatelessWidget {
  final bool showDivider;

  const BottomBackButton({
    this.showDivider = false,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Align(
      alignment: Alignment.bottomCenter,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (showDivider) const Divider(height: 1),
          SizedBox(
            height: context.isLandscape ? _kLandscapeButtonHeight : _kButtonHeight,
            width: double.infinity,
            child: Theme(
              data: context.theme.copyWith(
                textButtonTheme: TextButtonThemeData(
                  style: context.theme.textButtonTheme.style?.copyWith(
                    // Remove rounded edges
                    shape: const MaterialStatePropertyAll(RoundedRectangleBorder()),
                  ),
                ),
              ),
              child: TextIconButton(
                onPressed: () => Navigator.maybePop(context),
                iconPosition: IconPosition.start,
                icon: Icons.arrow_back,
                child: Text(context.l10n.generalBottomBackCta),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
