import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/pid/accept_offered_pid_usecase.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/pin_header.dart';
import '../../pin/bloc/pin_bloc.dart';
import '../../pin/pin_page.dart';

class RenewPidConfirmPinPage extends StatelessWidget {
  final OnPinValidatedCallback onPidAccepted;

  /// Callback for when accepting pid fails with an unrecoverable error.
  final OnPinErrorCallback onAcceptPidFailed;

  @visibleForTesting
  final PinBloc? bloc;

  const RenewPidConfirmPinPage({
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
        headerBuilder: (context, attempts, isFinalRound) => PinHeader(title: context.l10n.renewPidConfirmPinPageTitle),
        onPinValidated: onPidAccepted,
        onPinError: onAcceptPidFailed,
      ),
    );
  }
}
