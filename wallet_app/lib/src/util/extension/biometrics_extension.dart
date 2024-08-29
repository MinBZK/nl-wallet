import 'dart:io';

import 'package:flutter/cupertino.dart';

import '../../domain/usecase/biometrics/biometrics.dart';
import 'build_context_extension.dart';

extension BiometricsExtension on Biometrics {
  String prettyPrint(BuildContext context) {
    return switch (this) {
      Biometrics.face => Platform.isIOS ? context.l10n.biometricsFaceId : context.l10n.biometricsFace,
      Biometrics.fingerprint => Platform.isIOS ? context.l10n.biometricsTouchId : context.l10n.biometricsFingerprint,
      Biometrics.some =>
        Platform.isIOS ? context.l10n.biometricsFaceIdOrTouchId : context.l10n.biometricsFaceOrFingerprint,
      Biometrics.none => '',
    };
  }
}
