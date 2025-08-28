import 'package:flutter/material.dart';

import '../../../../../util/extension/build_context_extension.dart';
import '../../../../../util/extension/string_extension.dart';
import '../../utility/focus_on_init.dart';

/// Similar to the normal [BackButton] widget, but always uses the same icon (ios/android).
///
/// Additionally it always attempts to pull the accessibility focus towards itself, this is done
/// so that the a11y focus is reset to the top of the screen on page transitions. This behaviour
/// can be managed by setting [requestFocusOnInit]. Note that you might have to provide a [Key]
/// for this to behave as expected.
class BackIconButton extends StatelessWidget {
  final VoidCallback? onPressed;
  final bool requestFocusOnInit;

  const BackIconButton({
    this.onPressed,
    this.requestFocusOnInit = true,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return FocusOnInit(
      requestFocus: requestFocusOnInit,
      child: Semantics(
        button: true,
        onTap: onPressed ?? () => Navigator.pop(context),
        attributedLabel: context.l10n.generalWCAGBack.toAttributedString(context),
        excludeSemantics: true,
        child: Center(
          child: IconButton(
            onPressed: onPressed ?? () => Navigator.pop(context),
            icon: const Icon(Icons.arrow_back_rounded),
            tooltip: context.l10n.generalWCAGBack,
          ),
        ),
      ),
    );
  }
}
