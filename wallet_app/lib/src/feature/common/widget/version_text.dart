import 'package:flutter/material.dart';
import 'package:package_info_plus/package_info_plus.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';

class VersionText extends StatelessWidget {
  final TextStyle? prefixTextStyle;
  final TextStyle? valueTextStyle;

  const VersionText({
    this.prefixTextStyle,
    this.valueTextStyle,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<PackageInfo>(
      future: PackageInfo.fromPlatform(),
      builder: (context, snapshot) {
        if (snapshot.hasData) {
          final data = snapshot.data!;
          return Row(
            children: [
              Text.rich(
                context.l10n.generalVersionText.toTextSpan(context),
                style: prefixTextStyle ?? context.textTheme.bodyMedium,
              ),
              const SizedBox(width: 4),
              Text(
                '${data.version} (${data.buildNumber})',
                style: valueTextStyle ?? context.textTheme.bodyMedium,
              ),
            ],
          );
        } else {
          return const SizedBox.shrink();
        }
      },
    );
  }
}
