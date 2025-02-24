import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/biometrics/get_supported_biometrics_usecase.dart';
import '../../../util/extension/biometrics_extension.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';

class KeyboardBiometricKey extends StatelessWidget {
  final VoidCallback? onPressed;

  const KeyboardBiometricKey({
    this.onPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: MergeSemantics(
        child: FutureBuilder<Biometrics>(
          future: context.read<GetSupportedBiometricsUseCase>().invoke(),
          builder: (context, snapshot) {
            if (!snapshot.hasData) return const SizedBox.shrink();
            final biometrics = snapshot.data ?? Biometrics.none;
            return Semantics(
              keyboardKey: true,
              button: true,
              onTap: onPressed,
              attributedLabel: biometrics.prettyPrint(context).toAttributedString(context),
              child: TextButton.icon(
                onPressed: onPressed,
                label: const SizedBox.shrink(),
                style: context.theme.iconButtonTheme.style?.copyWith(
                  shape: WidgetStateProperty.all(
                    const RoundedRectangleBorder(borderRadius: BorderRadius.zero),
                  ),
                ),
                icon: Icon(biometrics.icon),
              ),
            );
          },
        ),
      ),
    );
  }
}
