import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/transfer/cancel_wallet_transfer_usecase.dart';
import '../../../domain/usecase/transfer/confirm_wallet_transfer_usecase.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/pin_header.dart';
import '../../pin/bloc/pin_bloc.dart';
import '../../pin/pin_page.dart';

class WalletTransferSourceConfirmPinPage extends StatelessWidget {
  final OnPinValidatedCallback onPinConfirmed;

  /// Callback for when confirming pin fails with an unrecoverable error.
  final OnPinErrorCallback onPinConfirmationFailed;

  const WalletTransferSourceConfirmPinPage({
    required this.onPinConfirmed,
    required this.onPinConfirmationFailed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    if (context.read<PinBloc?>() != null) {
      // BLoC provided, simply build so PinPage so we do not manage lifecycle of the BLoC. (fixes PVW-5185)
      return _buildPinPage();
    } else {
      // No BLoC provided, instantiate and manage internally.
      return BlocProvider<PinBloc>(
        create: (BuildContext context) => PinBloc(context.read<ConfirmWalletTransferUseCase>()),
        child: _buildPinPage(),
      );
    }
  }

  Widget _buildPinPage() {
    return PinPage(
      headerBuilder: (context, attempts, isFinalRound) =>
          PinHeader(title: context.l10n.walletTransferSourceConfirmPinPageTitle),
      onPinValidated: onPinConfirmed,
      onPinError: onPinConfirmationFailed,
      onStateChanged: (context, state) {
        if (state is PinValidateTimeout) context.read<CancelWalletTransferUseCase>().invoke();
        if (state is PinValidateBlocked) context.read<CancelWalletTransferUseCase>().invoke();
        return false;
      },
    );
  }
}
