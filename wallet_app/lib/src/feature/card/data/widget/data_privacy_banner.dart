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
    return InkWell(
      onTap: onPressed,
      child: Container(
        padding: const EdgeInsets.symmetric(vertical: 16, horizontal: 16),
        color: context.colorScheme.onSurface,
        child: Row(
          children: [
            Icon(
              Icons.gpp_maybe_outlined,
              color: context.colorScheme.background,
            ),
            const SizedBox(width: 8),
            Expanded(
              child: Text.rich(
                TextSpan(
                  text: context.l10n.cardDataScreenDataPrivacyBannerTitle.addSpaceSuffix,
                  style: context.textTheme.bodyMedium?.copyWith(
                    color: context.colorScheme.background,
                  ),
                  children: [
                    TextSpan(
                      text: context.l10n.cardDataScreenDataPrivacyBannerReadMore,
                      style: context.textTheme.bodyMedium?.copyWith(
                        color: context.colorScheme.background,
                        decoration: TextDecoration.underline,
                      ),
                    )
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
