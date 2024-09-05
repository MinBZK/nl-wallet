import 'dart:io';

import 'package:flutter/material.dart';

import '../../domain/usecase/biometrics/biometrics.dart';
import '../../wallet_icons.dart';
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

  IconData get icon {
    return switch (this) {
      Biometrics.face => Platform.isIOS ? WalletIcons.icon_face_id : Icons.face_unlock_outlined,
      Biometrics.fingerprint => Icons.fingerprint_outlined,
      Biometrics.some => Platform.isIOS ? WalletIcons.icon_face_id : Icons.fingerprint_outlined,
      Biometrics.none => Platform.isIOS ? WalletIcons.icon_face_id : Icons.fingerprint_outlined,
    };
  }
}
