import 'package:flutter/material.dart';

import '../../../../../environment.dart';
import '../../../../domain/model/result/application_error.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../wallet_core/error/core_error.dart';

class ApplicationErrorText extends StatelessWidget {
  final ApplicationError error;
  final TextStyle? prefixTextStyle;
  final TextStyle? valueTextStyle;
  final bool alignHorizontal;

  /// Whether to show the technical error details (type, description and data), instead of only the error type.
  ///
  /// Defaults to [Environment.showErrorDetails] when null.
  final bool? showTechnicalDetails;

  const ApplicationErrorText({
    required this.error,
    this.prefixTextStyle,
    this.valueTextStyle,
    this.alignHorizontal = true,
    this.showTechnicalDetails,
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
            text: (showTechnicalDetails ?? Environment.showErrorDetails) ? _describe(error) : '${error.runtimeType}',
            style: valueTextStyle ?? context.textTheme.bodyMedium,
          ),
        ],
      ),
    );
  }

  /// Describes the error including the wallet_core error info, as [Equatable.toString] only yields the type name.
  static String _describe(ApplicationError error) {
    final source = error.sourceError;
    if (source is! CoreError) return '${error.runtimeType}: $source';
    return [
      '${error.runtimeType} (${source.runtimeType})',
      if (source.description != null) source.description!,
      if (source.data != null) 'data: ${source.data}',
    ].join('\n');
  }
}
