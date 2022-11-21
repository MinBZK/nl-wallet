import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/usecase/pin/confirm_transaction_usecase.dart';
import '../../pin/bloc/pin_bloc.dart';
import '../../pin/pin_page.dart';

class VerificationConfirmPinPage extends StatelessWidget {
  final VoidCallback onPinValidated;

  const VerificationConfirmPinPage({
    required this.onPinValidated,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BlocProvider<PinBloc>(
      create: (BuildContext context) => PinBloc(context.read<ConfirmTransactionUseCase>(), context.read()),
      child: PinPage(
        headerBuilder: (context, attempts) {
          return Expanded(
            child: Container(
              width: double.infinity,
              padding: const EdgeInsets.all(16.0),
              child: attempts == null ? _buildNeutralHeader(context) : _buildErrorHeader(context, attempts),
            ),
          );
        },
        onPinValidated: onPinValidated,
      ),
    );
  }

  Widget _buildNeutralHeader(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        Text(
          locale.verificationConfirmPinPageTitle,
          style: Theme.of(context).textTheme.headline2,
        ),
        const SizedBox(height: 8),
        Text(
          locale.verificationConfirmPinPageDescription,
          style: Theme.of(context).textTheme.bodyText1,
        ),
      ],
    );
  }

  Widget _buildErrorHeader(BuildContext context, int attempts) {
    final locale = AppLocalizations.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        Text(
          locale.verificationConfirmPinPageErrorTitle,
          style: Theme.of(context).textTheme.headline2?.copyWith(color: Theme.of(context).errorColor),
        ),
        const SizedBox(height: 8),
        Text(
          locale.verificationConfirmPinPageErrorDescription(attempts),
          style: Theme.of(context).textTheme.bodyText1?.copyWith(color: Theme.of(context).errorColor),
        ),
      ],
    );
  }
}
