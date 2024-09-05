import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/biometrics/get_supported_biometrics_usecase.dart';
import '../../../util/extension/biometrics_extension.dart';
import '../../../util/extension/string_extension.dart';

class KeyboardBiometricKey extends StatelessWidget {
  final VoidCallback? onPressed;
  final Color? color;

  const KeyboardBiometricKey({
    this.onPressed,
    this.color,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: FutureBuilder<Biometrics>(
        future: context.read<GetSupportedBiometricsUseCase>().invoke(),
        builder: (context, snapshot) {
          if (!snapshot.hasData) return const SizedBox.shrink();
          final biometrics = snapshot.data ?? Biometrics.none;
          return Semantics(
            button: true,
            keyboardKey: true,
            attributedLabel: biometrics.prettyPrint(context).toAttributedString(context),
            child: InkWell(
              onTap: onPressed,
              child: Icon(
                biometrics.icon,
                color: color,
                size: 24,
              ),
            ),
          );
        },
      ),
    );
  }
}
