import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/pin/check_pin_usecase.dart';
import '../../../navigation/secured_page_route.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../pin/bloc/pin_bloc.dart';
import '../../pin/pin_page.dart';
import '../widget/button/icon/help_icon_button.dart';
import '../widget/pin_header.dart';
import '../widget/wallet_app_bar.dart';

class ConfirmWithPinScreen extends StatelessWidget {
  final Function(String) onPinValidated;

  @visibleForTesting
  final PinBloc? bloc;

  const ConfirmWithPinScreen({required this.onPinValidated, this.bloc, super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: const WalletAppBar(
        actions: [HelpIconButton()],
      ),
      body: BlocProvider<PinBloc>(
        create: (BuildContext context) => bloc ?? PinBloc(context.read<CheckPinUseCase>()),
        child: Builder(
          // Builder to make sure the onPinValidated callback can access the [PinBloc].
          builder: (context) {
            return PinPage(
              headerBuilder: (context, attempts, isFinalRound) {
                return PinHeader(title: context.l10n.generalConfirmWithPin);
              },
              onPinValidated: (_) => onPinValidated(context.read<PinBloc>().currentPin),
            );
          },
        ),
      ),
    );
  }

  /// Request PIN entry by the user, calling [onPinValidated] when a valid pin is provided.
  static Future<void> show(BuildContext context, OnPinValidatedCallback onPinValidated) {
    return Navigator.push(
      context,
      SecuredPageRoute(
        builder: (c) => ConfirmWithPinScreen(onPinValidated: onPinValidated),
      ),
    );
  }
}
