import 'dart:async';

import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/usecase/biometrics/biometric_authentication_result.dart';
import '../../domain/usecase/biometrics/is_biometric_login_enabled_usecase.dart';
import '../../domain/usecase/biometrics/unlock_wallet_with_biometrics_usecase.dart';
import '../common/dialog/locked_out_dialog.dart';
import '../common/widget/button/icon/info_icon_button.dart';
import '../common/widget/utility/auto_biometric_unlock_trigger.dart';
import '../common/widget/wallet_app_bar.dart';
import 'pin_page.dart';

class PinScreen extends StatelessWidget {
  final VoidCallback onUnlock;

  const PinScreen({required this.onUnlock, super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('pinScreen'),
      appBar: const WalletAppBar(
        automaticallyImplyLeading: false,
        actions: [InfoIconButton()],
        fadeInTitleOnScroll: false,
      ),
      body: FutureBuilder<bool>(
        future: context.read<IsBiometricLoginEnabledUseCase>().invoke(),
        initialData: false,
        builder: (context, snapshot) {
          final showBiometricUnlock = snapshot.data ?? false;
          return AutoBiometricUnlockTrigger(
            onTriggerBiometricUnlock: _performBiometricUnlock,
            child: PinPage(
              onPinValidated: (_) => onUnlock,
              showTopDivider: true,
              onBiometricUnlockRequested: showBiometricUnlock ? () => _performBiometricUnlock(context) : null,
            ),
          );
        },
      ),
    );
  }

  Future<void> _performBiometricUnlock(BuildContext context) async {
    final unlockWithBiometricsUseCase = context.read<UnlockWalletWithBiometricsUseCase>();
    final unlockResult = await unlockWithBiometricsUseCase.invoke();
    await unlockResult.process(
      onSuccess: (authResult) {
        switch (authResult) {
          case BiometricAuthenticationResult.success:
            onUnlock();
          case BiometricAuthenticationResult.lockedOut:
            unawaited(LockedOutDialog.show(context));
          case BiometricAuthenticationResult.setupRequired:
            Fimber.d('Authentication failed $unlockResult');
        }
      },
      onError: (error) => Fimber.e('Failed to unlock wallet with biometrics', ex: error),
    );
  }
}
