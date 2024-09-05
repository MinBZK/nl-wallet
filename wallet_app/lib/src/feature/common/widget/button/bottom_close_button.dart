import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';
import 'button_content.dart';
import 'list_button.dart';

/// Close button that is aligned at the bottom of the screen, rendered with a divider.
/// Often used in Sheets.
class BottomCloseButton extends StatelessWidget {
  const BottomCloseButton({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Align(
      alignment: Alignment.bottomCenter,
      child: ListButton(
        onPressed: () => Navigator.maybePop(context),
        icon: const Icon(Icons.close_outlined),
        mainAxisAlignment: MainAxisAlignment.center,
        iconPosition: IconPosition.start,
        dividerSide: DividerSide.top,
        text: Text.rich(context.l10n.generalSheetCloseCta.toTextSpan(context)),
      ),
    );
  }
}
