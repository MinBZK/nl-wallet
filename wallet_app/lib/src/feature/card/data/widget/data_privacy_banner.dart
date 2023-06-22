import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';

class DataPrivacyBanner extends StatelessWidget {
  final VoidCallback? onPressed;

  const DataPrivacyBanner({
    required this.onPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 16, horizontal: 16),
      color: context.colorScheme.tertiaryContainer,
      child: Row(
        children: [
          const Icon(Icons.gpp_maybe_outlined),
          const SizedBox(width: 8),
          Expanded(
            child: Text.rich(
              TextSpan(
                text: context.l10n.cardDataScreenDataPrivacyBannerTitle.addSpaceSuffix,
                children: [
                  TextSpan(
                    text: context.l10n.cardDataScreenDataPrivacyBannerReadMore,
                    recognizer: TapGestureRecognizer()..onTap = onPressed,
                    style: context.textTheme.bodyMedium?.copyWith(decoration: TextDecoration.underline),
                  )
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
