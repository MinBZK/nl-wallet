import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
import '../../../common/widget/text/title_text.dart';

class QrScannerHint extends StatelessWidget {
  const QrScannerHint({super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(vertical: 8),
      margin: EdgeInsets.only(top: context.mediaQuery.padding.top),
      color: context.theme.appBarTheme.backgroundColor?.withValues(alpha: 0.9),
      child: TitleText(
        context.l10n.qrScreenScanHint,
        textAlign: TextAlign.center,
        style: context.textTheme.bodyLarge,
      ),
    );
  }
}
