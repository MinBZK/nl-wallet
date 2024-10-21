import 'package:flutter/material.dart';

import '../../../../theme/wallet_theme.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../common/widget/text/body_text.dart';

class DataPrivacyBanner extends StatelessWidget {
  const DataPrivacyBanner({super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        border: Border.all(color: context.colorScheme.error, width: 1),
        borderRadius: WalletTheme.kBorderRadius12,
      ),
      child: Row(
        children: [
          Expanded(
            child: BodyText(context.l10n.cardDataScreenDataPrivacyBanner),
          ),
          const SizedBox(width: 20),
          SizedBox(
            height: 24,
            width: 24,
            child: Icon(
              Icons.back_hand_outlined,
              size: 24,
              color: context.colorScheme.error,
            ),
          ),
        ],
      ),
    );
  }
}
