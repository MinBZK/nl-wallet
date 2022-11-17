import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/usecase/pin/confirm_transaction_usecase.dart';
import '../../wallet_routes.dart';
import 'bloc/pin_bloc.dart';
import 'pin_screen.dart';

/// Pin prompt that can be shown at any time to request and verify the user's pin using the static [confirm] method.
class PinPrompt extends StatelessWidget {
  const PinPrompt({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BlocProvider<PinBloc>(
      create: (BuildContext context) => PinBloc(context.read<ConfirmTransactionUseCase>(), context.read()),
      child: PinScreen(
        onUnlock: () => Navigator.pop(context, true),
      ),
    );
  }

  static Future<bool> confirm(BuildContext context) async {
    final result = await Navigator.pushNamed(context, WalletRoutes.confirmRoute);
    return result == true;
  }
}
