import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
import '../flutter_app_configuration_provider.dart';

class ConfigVersionText extends StatelessWidget {
  final TextStyle? prefixTextStyle;
  final TextStyle? valueTextStyle;
  final bool alignHorizontal;

  const ConfigVersionText({
    this.prefixTextStyle,
    this.valueTextStyle,
    this.alignHorizontal = true,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return FlutterAppConfigurationProvider(
      builder: (config) => Text.rich(
        TextSpan(
          children: [
            TextSpan(
              text: context.l10n.generalConfigVersionText,
              style: prefixTextStyle ?? context.textTheme.bodyMedium,
            ),
            alignHorizontal ? const TextSpan(text: ' ') : const TextSpan(text: '\n'),
            TextSpan(
              text: config.version.toString(),
              style: valueTextStyle ?? context.textTheme.bodyMedium,
            ),
          ],
        ),
      ),
    );
  }
}
