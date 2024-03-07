import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
import 'bottom_button.dart';
import 'text_icon_button.dart';

/// Back button that is aligned at the bottom of the screen,
/// rendered with a divider.
/// Often used as a direct child of a [SliverFillRemaining] widget.
class BottomBackButton extends StatelessWidget {
  const BottomBackButton({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return BottomButton(
      button: TextIconButton(
        onPressed: () => Navigator.maybePop(context),
        iconPosition: IconPosition.start,
        icon: Icons.arrow_back,
        child: Text(context.l10n.generalBottomBackCta),
      ),
    );
  }
}
