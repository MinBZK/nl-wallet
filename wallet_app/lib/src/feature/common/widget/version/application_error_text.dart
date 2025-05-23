import 'package:flutter/material.dart';

import '../../../../domain/model/result/application_error.dart';
import '../../../../util/extension/build_context_extension.dart';

class ApplicationErrorText extends StatelessWidget {
  final ApplicationError error;
  final TextStyle? prefixTextStyle;
  final TextStyle? valueTextStyle;
  final bool alignHorizontal;

  const ApplicationErrorText({
    required this.error,
    this.prefixTextStyle,
    this.valueTextStyle,
    this.alignHorizontal = true,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Text.rich(
      TextSpan(
        children: [
          TextSpan(
            text: context.l10n.applicationErrorDetailsTitle,
            style: prefixTextStyle ?? context.textTheme.bodyMedium,
          ),
          alignHorizontal ? const TextSpan(text: ' ') : const TextSpan(text: '\n'),
          TextSpan(
            text: error.toString(),
            style: valueTextStyle ?? context.textTheme.bodyMedium,
          ),
        ],
      ),
    );
  }
}
