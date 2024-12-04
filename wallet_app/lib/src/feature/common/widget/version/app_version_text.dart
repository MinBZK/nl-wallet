import 'package:flutter/material.dart';
import 'package:package_info_plus/package_info_plus.dart';

import '../../../../util/extension/build_context_extension.dart';

class AppVersionText extends StatelessWidget {
  final TextStyle? prefixTextStyle;
  final TextStyle? valueTextStyle;
  final bool alignHorizontal;

  const AppVersionText({
    this.prefixTextStyle,
    this.valueTextStyle,
    this.alignHorizontal = true,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<PackageInfo>(
      future: PackageInfo.fromPlatform(),
      builder: (context, snapshot) {
        if (snapshot.hasData) {
          final data = snapshot.data!;
          return Text.rich(
            TextSpan(
              children: [
                TextSpan(
                  text: context.l10n.generalVersionText,
                  style: prefixTextStyle ?? context.textTheme.bodyMedium,
                ),
                alignHorizontal ? const TextSpan(text: ' ') : const TextSpan(text: '\n'),
                TextSpan(
                  text: '${data.version} (${data.buildNumber})',
                  style: valueTextStyle ?? context.textTheme.bodyMedium,
                ),
              ],
            ),
          );
        } else {
          return const SizedBox.shrink();
        }
      },
    );
  }
}
