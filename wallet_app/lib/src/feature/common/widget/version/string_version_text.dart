import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/model/result/result.dart';
import '../../../../domain/usecase/version/get_version_string_usecase.dart';
import '../../../../util/extension/build_context_extension.dart';

class StringVersionText extends StatelessWidget {
  final TextStyle? prefixTextStyle;
  final TextStyle? valueTextStyle;
  final bool alignHorizontal;

  const StringVersionText({
    this.prefixTextStyle,
    this.valueTextStyle,
    this.alignHorizontal = true,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<Result<String>>(
      future: context.read<GetVersionStringUseCase>().invoke(),
      builder: (context, snapshot) {
        if (!snapshot.hasData) return const SizedBox.shrink();
        final versionResult = snapshot.data!;
        if (versionResult.hasError) return const SizedBox.shrink();
        return Text.rich(
          TextSpan(
            children: [
              TextSpan(
                text: context.l10n.generalVersionText,
                style: prefixTextStyle ?? context.textTheme.bodyMedium,
              ),
              alignHorizontal ? const TextSpan(text: ' ') : const TextSpan(text: '\n'),
              TextSpan(
                text: versionResult.value,
                style: valueTextStyle ?? context.textTheme.bodyMedium,
              ),
            ],
          ),
        );
      },
    );
  }
}
