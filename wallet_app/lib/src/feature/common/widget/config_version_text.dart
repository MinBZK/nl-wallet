import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import 'flutter_app_configuration_provider.dart';

class ConfigVersionText extends StatelessWidget {
  final TextStyle? textStyle;

  const ConfigVersionText({this.textStyle, super.key});

  @override
  Widget build(BuildContext context) {
    return FlutterAppConfigurationProvider(
        builder: (config) => Text(
              context.l10n.generalConfigVersionText(
                config.version,
              ),
              style: textStyle ?? context.textTheme.bodyMedium,
            ));
  }
}
