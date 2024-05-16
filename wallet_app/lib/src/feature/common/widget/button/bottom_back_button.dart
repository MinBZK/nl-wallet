import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
import 'button_content.dart';
import 'list_button.dart';

/// Back button that is aligned at the bottom of the screen, rendered with a divider.
/// Often used as a direct child of a [SliverFillRemaining] widget.
class BottomBackButton extends StatelessWidget {
  const BottomBackButton({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Align(
      alignment: Alignment.bottomCenter,
      child: ListButton(
        onPressed: () => Navigator.maybePop(context),
        icon: const Icon(Icons.arrow_back),
        mainAxisAlignment: MainAxisAlignment.center,
        iconPosition: IconPosition.start,
        dividerSide: DividerSide.top,
        text: Text(context.l10n.generalBottomBackCta),
      ),
    );
  }
}
