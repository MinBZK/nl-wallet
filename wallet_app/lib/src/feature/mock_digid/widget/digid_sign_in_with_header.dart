// coverage:ignore-file
import 'package:flutter/material.dart';

import '../../../theme/base_wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/text/title_text.dart';

class DigidSignInWithHeader extends StatelessWidget {
  const DigidSignInWithHeader({super.key});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          TitleText(
            context.l10n.mockDigidScreenHeaderTitle,
            style: context.textTheme.headlineLarge?.copyWith(color: context.colorScheme.primary),
          ),
          const SizedBox(height: 8),
          Text(
            context.l10n.mockDigidScreenHeaderSubtitle,
            style: context.textTheme.bodyMedium?.copyWith(fontVariations: [BaseWalletTheme.fontVariationBold]),
          ),
        ],
      ),
    );
  }
}
