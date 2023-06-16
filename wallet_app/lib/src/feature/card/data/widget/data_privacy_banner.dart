import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../../util/extension/string_extension.dart';

class DataPrivacyBanner extends StatelessWidget {
  final VoidCallback? onPressed;

  const DataPrivacyBanner({
    required this.onPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 16, horizontal: 16),
      color: Theme.of(context).colorScheme.tertiaryContainer,
      child: Row(
        children: [
          const Icon(Icons.gpp_maybe_outlined),
          const SizedBox(width: 8),
          Expanded(
            child: Text.rich(
              TextSpan(
                text: locale.cardDataScreenDataPrivacyBannerTitle.addSpaceSuffix,
                children: [
                  TextSpan(
                    text: locale.cardDataScreenDataPrivacyBannerReadMore,
                    recognizer: TapGestureRecognizer()..onTap = onPressed,
                    style: Theme.of(context).textTheme.bodyMedium?.copyWith(decoration: TextDecoration.underline),
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
