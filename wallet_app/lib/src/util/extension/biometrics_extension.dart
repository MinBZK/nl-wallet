import 'package:flutter/material.dart';

import '../../domain/usecase/biometrics/biometrics.dart';
import '../../wallet_icons.dart';
import 'build_context_extension.dart';

extension BiometricsExtension on Biometrics {
  String prettyPrint(BuildContext context) {
    final bool isIos = context.theme.platform == TargetPlatform.iOS;
    return switch (this) {
      Biometrics.face => isIos ? context.l10n.biometricsFaceId : context.l10n.biometricsFace,
      Biometrics.fingerprint => isIos ? context.l10n.biometricsTouchId : context.l10n.biometricsFingerprint,
      Biometrics.some => isIos ? context.l10n.biometricsFaceIdOrTouchId : context.l10n.biometricsFaceOrFingerprint,
      Biometrics.none => '',
    };
  }

  IconData icon(BuildContext context) {
    final bool isIos = context.theme.platform == TargetPlatform.iOS;
    return switch (this) {
      Biometrics.face => isIos ? WalletIcons.icon_face_id : Icons.face_unlock_outlined,
      Biometrics.fingerprint => Icons.fingerprint_outlined,
      Biometrics.some => isIos ? WalletIcons.icon_face_id : Icons.fingerprint_outlined,
      Biometrics.none => isIos ? WalletIcons.icon_face_id : Icons.fingerprint_outlined,
    };
  }
}
