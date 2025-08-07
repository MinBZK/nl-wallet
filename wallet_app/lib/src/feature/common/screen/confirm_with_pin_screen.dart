import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/pin/check_pin_usecase.dart';
import '../../../navigation/secured_page_route.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../pin/bloc/pin_bloc.dart';
import '../../pin/pin_page.dart';
import '../mixin/lock_state_mixin.dart';
import '../widget/button/icon/help_icon_button.dart';
import '../widget/pin_header.dart';
import '../widget/wallet_app_bar.dart';

class ConfirmWithPinScreen extends StatefulWidget {
  final Function(String) onPinValidated;

  @visibleForTesting
  final PinBloc? bloc;

  const ConfirmWithPinScreen({required this.onPinValidated, this.bloc, super.key});

  @override
  State<ConfirmWithPinScreen> createState() => _ConfirmWithPinScreenState();

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

class _ConfirmWithPinScreenState extends State<ConfirmWithPinScreen> with LockStateMixin<ConfirmWithPinScreen> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: const WalletAppBar(
        actions: [HelpIconButton()],
        fadeInTitleOnScroll: false,
      ),
      body: BlocProvider<PinBloc>(
        create: (BuildContext context) => widget.bloc ?? PinBloc(context.read<CheckPinUseCase>()),
        child: Builder(
          // Builder to make sure the onPinValidated callback can access the [PinBloc].
          builder: (context) {
            return PinPage(
              headerBuilder: (context, attempts, isFinalRound) {
                return PinHeader(title: context.l10n.generalConfirmWithPin);
              },
              onPinValidated: (_) => widget.onPinValidated(context.read<PinBloc>().currentPin),
            );
          },
        ),
      ),
    );
  }

  @override
  void onLock() {
    /// Pop the ConfirmWithPinScreen until the dashboard
    Navigator.popUntil(context, ModalRoute.withName(WalletRoutes.dashboardRoute));
  }

  @override
  void onUnlock() {
    /* unused */
  }
}
