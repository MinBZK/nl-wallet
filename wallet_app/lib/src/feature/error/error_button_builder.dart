import 'package:flutter/material.dart';

import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../common/sheet/error_details_sheet.dart';
import '../common/widget/button/confirm/confirm_buttons.dart';
import '../common/widget/button/primary_button.dart';
import '../common/widget/button/tertiary_button.dart';
import 'error_cta_style.dart';

export 'error_cta_style.dart';

class ErrorButtonBuilder {
  ErrorButtonBuilder._();

  static FitsWidthWidget buildPrimaryButtonFor(
    BuildContext context,
    ErrorCtaStyle style, {
    VoidCallback? onPressed,
    String? cta,
  }) {
    return switch (style) {
      ErrorCtaStyle.retry => PrimaryButton(
          text: Text.rich((cta ?? context.l10n.generalRetry).toTextSpan(context)),
          icon: const Icon(Icons.replay_outlined),
          onPressed: onPressed ?? () => Navigator.maybePop(context),
        ),
      ErrorCtaStyle.close => PrimaryButton(
          text: Text.rich((cta ?? context.l10n.generalClose).toTextSpan(context)),
          icon: const Icon(Icons.close_outlined),
          onPressed: onPressed ?? () => Navigator.maybePop(context),
        ),
    };
  }

  static FitsWidthWidget buildShowDetailsButton(BuildContext context) {
    return TertiaryButton(
      text: Text.rich(context.l10n.generalShowDetailsCta.toTextSpan(context)),
      icon: const Icon(Icons.info_outline_rounded),
      onPressed: () => ErrorDetailsSheet.show(context),
    );
  }
}
