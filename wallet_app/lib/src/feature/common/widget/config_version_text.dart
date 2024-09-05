import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import 'flutter_app_configuration_provider.dart';

class ConfigVersionText extends StatelessWidget {
  final TextStyle? prefixTextStyle;
  final TextStyle? valueTextStyle;

  const ConfigVersionText({
    this.prefixTextStyle,
    this.valueTextStyle,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return FlutterAppConfigurationProvider(
      builder: (config) => Row(
        children: [
          Text.rich(
            context.l10n.generalConfigVersionText.toTextSpan(context),
            style: prefixTextStyle ?? context.textTheme.bodyMedium,
          ),
          const SizedBox(width: 4),
          Text(
            config.version.toString(),
            style: valueTextStyle ?? context.textTheme.bodyMedium,
          ),
        ],
      ),
    );
  }
}
