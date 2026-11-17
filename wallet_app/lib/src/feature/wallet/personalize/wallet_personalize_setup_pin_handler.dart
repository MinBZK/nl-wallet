import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../pin/bloc/pin_bloc.dart';
import '../../pin/pin_page.dart';
import 'wallet_personalize_setup_failed_screen.dart';

/// Provides the contextualized pin handling for the pin entries in the personalize flow, i.e. when the user confirms
/// their (already chosen) pin to retrieve or accept the pid as part of the initial wallet setup.
/// Returns true when the state was handled.
bool handleSetupPinState(BuildContext context, PinState state) {
  /// PVW-2759: Edge case where we want to provide contextualized error copy during the initial setup.
  if (state is PinValidateFailure && state.attemptsLeftInRound == 1) {
    PinPage.showPinErrorDialog(
      context,
      context.l10n.pinErrorDialogRestartSetupWarning,
    );
    return true;
  }

  /// PVW-1037 (criteria 6): Handle the special case where the user has forgotten her pin during initial setup.
  final bool forcedReset = state is PinValidateTimeout || state is PinValidateBlocked;
  if (forcedReset) WalletPersonalizeSetupFailedScreen.show(context);
  return forcedReset;
}
