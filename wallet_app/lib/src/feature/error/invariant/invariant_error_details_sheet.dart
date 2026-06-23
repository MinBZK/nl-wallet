import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../theme/base_wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/button/bottom_close_button.dart';
import '../../common/widget/button/secondary_button.dart';
import '../../common/widget/button/tertiary_button.dart';
import '../../common/widget/text/body_text.dart';
import '../../common/widget/text/title_text.dart';
import '../../common/widget/wallet_scrollbar.dart';

/// Bottom sheet showing the technical error [code] of an invariant violation.
///
/// The "Copy" action is only available in developer (debug) builds.
class InvariantErrorDetailsSheet extends StatelessWidget {
  final String? code;

  /// Whether the developer-only copy action is shown. Defaults to [kDebugMode];
  /// only overridden in tests to golden the release (non-debug) rendering.
  @visibleForTesting
  final bool showCopyButton;

  const InvariantErrorDetailsSheet({
    this.code,
    this.showCopyButton = kDebugMode,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
            child: Column(
              children: [
                TitleText(context.l10n.invariantErrorDetailsSheetTitle),
                const SizedBox(height: 16),
                BodyText(context.l10n.invariantErrorDetailsSheetDescription),
                const SizedBox(height: 24),
                TitleText(
                  context.l10n.invariantErrorDetailsSheetCodeLabel,
                  style: context.textTheme.bodyLarge?.copyWith(fontVariations: [BaseWalletTheme.fontVariationBold]),
                ),
                // TODO(PVW-5921): provide the real error code, currently showing a preview code.
                BodyText(code ?? '-'),
              ],
            ),
          ),
          // Copy is developer-only, so it's only built in debug builds. When present, Copy and Close
          // share one padded column so the two stacked buttons keep a consistent rhythm.
          if (showCopyButton) ...[
            const Divider(),
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 24, 16, 24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  SecondaryButton(
                    text: Text(context.l10n.invariantErrorDetailsSheetCopyCta),
                    icon: const Icon(Icons.copy_outlined),
                    onPressed: () => unawaited(Clipboard.setData(ClipboardData(text: code ?? ''))),
                  ),
                  const SizedBox(height: 12),
                  TertiaryButton(
                    text: Text(context.l10n.generalSheetCloseCta),
                    icon: const Icon(Icons.close_outlined),
                    onPressed: () => Navigator.maybePop(context),
                  ),
                ],
              ),
            ),
          ] else
            const BottomCloseButton(),
        ],
      ),
    );
  }

  static Future<void> show(BuildContext context, {String? code}) async {
    return showModalBottomSheet<void>(
      context: context,
      isDismissible: !context.isScreenReaderEnabled, // Avoid announcing the scrim
      isScrollControlled: true,
      builder: (BuildContext context) {
        return WalletScrollbar(
          child: SingleChildScrollView(
            child: InvariantErrorDetailsSheet(code: code),
          ),
        );
      },
    );
  }
}
