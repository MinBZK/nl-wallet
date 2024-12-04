import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

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
    return FutureBuilder<String>(
      future: context.read<GetVersionStringUseCase>().invoke(),
      builder: (context, snapshot) {
        if (snapshot.hasData) {
          final versionString = snapshot.data!;
          return Text.rich(
            TextSpan(
              children: [
                TextSpan(
                  text: context.l10n.generalVersionText,
                  style: prefixTextStyle ?? context.textTheme.bodyMedium,
                ),
                alignHorizontal ? const TextSpan(text: ' ') : const TextSpan(text: '\n'),
                TextSpan(
                  text: versionString,
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
