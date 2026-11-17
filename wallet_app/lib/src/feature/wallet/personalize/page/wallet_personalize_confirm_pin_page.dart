import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:material_ui/material_ui.dart';

import '../../../../data/repository/pid/pid_repository.dart';
import '../../../../domain/usecase/pid/accept_offered_pid_usecase.dart';
import '../../../../domain/usecase/pid/commit_issued_pid_usecase.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../common/widget/pin_header.dart';
import '../../../pin/bloc/pin_bloc.dart';
import '../../../pin/pin_page.dart';
import '../wallet_personalize_setup_pin_handler.dart';

class WalletPersonalizeConfirmPinPage extends StatelessWidget {
  final OnPinValidatedCallback<TransferState> onPidAccepted;

  /// Callback for when accepting pid fails with an unrecoverable error.
  final OnPinErrorCallback onAcceptPidFailed;

  /// True when the pid was already issued for review, in which case the PIN is used to commit it to the wallet.
  final bool issuedForReview;

  @visibleForTesting
  final PinBloc? bloc;

  const WalletPersonalizeConfirmPinPage({
    required this.onPidAccepted,
    required this.onAcceptPidFailed,
    this.issuedForReview = false,
    this.bloc,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return BlocProvider<PinBloc>(
      create: (BuildContext context) =>
          bloc ??
          PinBloc(issuedForReview ? context.read<CommitIssuedPidUseCase>() : context.read<AcceptOfferedPidUseCase>()),
      child: PinPage(
        key: const Key('personalizeConfirmPinPage'),
        headerBuilder: (context, attempts, isFinalRound) {
          return PinHeader(title: context.l10n.walletPersonalizeConfirmPinPageTitle);
        },
        onPinValidated: (result) => onPidAccepted(result as TransferState),
        onPinError: onAcceptPidFailed,
        onStateChanged: handleSetupPinState,
      ),
    );
  }
}
