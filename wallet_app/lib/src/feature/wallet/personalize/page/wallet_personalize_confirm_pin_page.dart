import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../domain/usecase/pid/accept_offered_pid_usecase.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../common/widget/pin_header.dart';
import '../../../pin/bloc/pin_bloc.dart';
import '../../../pin/pin_page.dart';
import '../wallet_personalize_setup_failed_screen.dart';

class WalletPersonalizeConfirmPinPage extends StatelessWidget {
  final OnPinValidatedCallback onPidAccepted;

  /// Callback for when accepting pid fails with an unrecoverable error.
  final OnPinErrorCallback onAcceptPidFailed;

  @visibleForTesting
  final PinBloc? bloc;

  const WalletPersonalizeConfirmPinPage({
    required this.onPidAccepted,
    required this.onAcceptPidFailed,
    this.bloc,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return BlocProvider<PinBloc>(
      create: (BuildContext context) => bloc ?? PinBloc(context.read<AcceptOfferedPidUseCase>()),
      child: PinPage(
        key: const Key('personalizeConfirmPinPage'),
        headerBuilder: (context, attempts, isFinalAttempt) {
          return PinHeader(title: context.l10n.walletPersonalizeConfirmPinPageTitle);
        },
        onPinValidated: onPidAccepted,
        onPinError: onAcceptPidFailed,
        onStateChanged: (context, state) {
          /// Handle the special case where the user has forgotten her pin right after the setup
          if (state is PinValidateTimeout) WalletPersonalizeSetupFailedScreen.show(context);
          return state is PinValidateTimeout;
        },
      ),
    );
  }
}
