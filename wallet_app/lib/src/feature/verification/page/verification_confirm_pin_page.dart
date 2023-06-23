import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/pin/confirm_transaction_usecase.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/pin_header.dart';
import '../../pin/bloc/pin_bloc.dart';
import '../../pin/pin_page.dart';

class VerificationConfirmPinPage extends StatelessWidget {
  final VoidCallback onPinValidated;

  @visibleForTesting
  final PinBloc? bloc;

  const VerificationConfirmPinPage({
    required this.onPinValidated,
    this.bloc,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BlocProvider<PinBloc>(
      create: (BuildContext context) => bloc ?? PinBloc(context.read<ConfirmTransactionUseCase>()),
      child: PinPage(
        headerBuilder: (context, attempts, isFinalAttempt) {
          final hasError = attempts != null;
          final String title, description;
          if (!hasError) {
            title = context.l10n.verificationConfirmPinPageTitle;
            description = context.l10n.verificationConfirmPinPageDescription;
          } else {
            title = context.l10n.verificationConfirmPinPageErrorTitle;
            description = context.l10n.verificationConfirmPinPageErrorDescription(attempts);
          }
          return PinHeader(
            hasError: hasError,
            title: title,
            description: description,
          );
        },
        onPinValidated: onPinValidated,
      ),
    );
  }
}
