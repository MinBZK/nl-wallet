import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../wallet_constants.dart';
import '../../wallet_routes.dart';
import '../common/widget/text_arrow_button.dart';
import 'bloc/pin_bloc.dart';
import 'widget/pin_field.dart';
import 'widget/pin_keyboard.dart';

class PinScreen extends StatelessWidget {
  final VoidCallback? onUnlock;

  const PinScreen({this.onUnlock, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BlocListener<PinBloc, PinState>(
      listener: (context, state) {
        if (state is PinValidateSuccess) onUnlock?.call();
        if (state is PinValidateBlocked) Navigator.restorablePushReplacementNamed(context, WalletRoutes.splashRoute);
      },
      child: Scaffold(
        appBar: AppBar(
          title: Text(AppLocalizations.of(context).pinScreenTitle),
          centerTitle: true,
        ),
        body: Column(
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            const Spacer(),
            const FlutterLogo(size: 80),
            const SizedBox(height: 24),
            _buildTextHeader(),
            const SizedBox(height: 24),
            _buildPinField(),
            const SizedBox(height: 24),
            _buildForgotCodeButton(),
            const Spacer(),
            _buildPinKeyboard(),
          ],
        ),
      ),
    );
  }

  Widget _buildTextHeader() {
    return BlocBuilder<PinBloc, PinState>(
      builder: (context, state) {
        if (state is PinValidateFailure) {
          return Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                AppLocalizations.of(context).pinScreenErrorHeader,
                style: Theme.of(context).textTheme.headline3?.copyWith(color: Theme.of(context).errorColor),
                textAlign: TextAlign.center,
              ),
              Text(
                AppLocalizations.of(context).pinScreenAttemptsCount(state.leftoverAttempts),
                style: Theme.of(context).textTheme.bodyText1?.copyWith(color: Theme.of(context).errorColor),
                textAlign: TextAlign.center,
              ),
            ],
          );
        } else {
          return Column(
            children: [
              Text(
                AppLocalizations.of(context).pinScreenHeader,
                style: Theme.of(context).textTheme.headline3,
                textAlign: TextAlign.center,
              ),
              Text(
                '',
                style: Theme.of(context).textTheme.bodyText1,
                textAlign: TextAlign.center,
              ),
            ],
          );
        }
      },
    );
  }

  Widget _buildPinField() {
    return BlocBuilder<PinBloc, PinState>(
      builder: (context, state) {
        return PinField(
          digits: kPinDigits,
          enteredDigits: _resolveEnteredDigits(state),
        );
      },
    );
  }

  Widget _buildForgotCodeButton() {
    return BlocBuilder<PinBloc, PinState>(
      builder: (context, state) {
        final buttonEnabled = state is PinEntryInProgress || state is PinValidateFailure;
        return TextArrowButton(
          onPressed: buttonEnabled ? () => Fimber.d('TODO: Navigate to forgot pin route') : null,
          child: Text(AppLocalizations.of(context).pinScreenForgotPinCta),
        );
      },
    );
  }

  Widget _buildPinKeyboard() {
    return BlocBuilder<PinBloc, PinState>(
      builder: (context, state) {
        return AnimatedOpacity(
          duration: kDefaultAnimationDuration,
          opacity: state is PinValidateInProgress ? 0.3 : 1,
          child: PinKeyboard(
            onKeyPressed:
                _digitKeysEnabled(state) ? (digit) => context.read<PinBloc>().add(PinDigitPressed(digit)) : null,
            onBackspacePressed:
                _backspaceKeyEnabled(state) ? () => context.read<PinBloc>().add(const PinBackspacePressed()) : null,
          ),
        );
      },
    );
  }

  bool _digitKeysEnabled(PinState state) {
    if (state is PinEntryInProgress) return true;
    if (state is PinValidateFailure) return true;
    return false;
  }

  bool _backspaceKeyEnabled(PinState state) {
    if (state is PinEntryInProgress) return true;
    if (state is PinValidateFailure) return true;
    return false;
  }

  int _resolveEnteredDigits(PinState state) {
    if (state is PinEntryInProgress) return state.enteredDigits;
    if (state is PinValidateInProgress) return kPinDigits;
    if (state is PinValidateSuccess) return kPinDigits;
    if (state is PinValidateFailure) return kPinDigits;
    return 0;
  }
}
