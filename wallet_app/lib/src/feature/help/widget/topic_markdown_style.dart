import 'package:flutter_markdown_plus/flutter_markdown_plus.dart';
import 'package:material_ui/material_ui.dart';

import '../../../theme/base_wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';

MarkdownStyleSheet topicMarkdownStyleSheet(BuildContext context) {
  return MarkdownStyleSheet(
    textScaler: context.textScaler,
    p: context.textTheme.bodyLarge,
    a: context.textTheme.bodyLarge?.copyWith(
      fontVariations: const [BaseWalletTheme.fontVariationRegular],
      decoration: TextDecoration.underline,
      color: context.colorScheme.primary,
    ),
  );
}
