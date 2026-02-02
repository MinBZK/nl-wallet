import 'package:flutter/material.dart';

import '../../../theme/wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';

const _kRevocationCodeDisplaySeparator = '-';

class RevocationCodeText extends StatelessWidget {
  final String revocationCode;

  String get displayCode => _chunkString(revocationCode).join(_kRevocationCodeDisplaySeparator);

  const RevocationCodeText({required this.revocationCode, super.key});

  @override
  Widget build(BuildContext context) {
    final textStyle = context.textTheme.headlineSmall;
    return Material(
      color: context.colorScheme.tertiaryContainer,
      shape: RoundedRectangleBorder(
        side: BorderSide(width: 1, color: context.colorScheme.outlineVariant),
        borderRadius: WalletTheme.kBorderRadius12,
      ),
      child: InkWell(
        borderRadius: WalletTheme.kBorderRadius12,
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 16),
          child: Center(
            child: Wrap(
              alignment: .center,
              children: displayCode.characters.map(
                (char) {
                  return Semantics(
                    container: char != _kRevocationCodeDisplaySeparator,
                    child: Text.rich(char.toTextSpan(context), style: textStyle),
                  );
                },
              ).toList(),
            ),
          ),
        ),
      ),
    );
  }

  // Split a string into chunks of a specified size
  List<String> _chunkString(String input, {int chunkSize = 4}) {
    final chunks = <String>[];
    for (int i = 0; i < input.length; i += chunkSize) {
      chunks.add(input.substring(i, (i + chunkSize).clamp(0, input.length)));
    }
    return chunks;
  }
}
