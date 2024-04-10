import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/sign/accept_sign_agreement_usecase.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/pin_header.dart';
import '../../pin/bloc/pin_bloc.dart';
import '../../pin/pin_page.dart';

class SignConfirmPinPage extends StatelessWidget {
  final OnPinValidatedCallback onPinValidated;

  @visibleForTesting
  final PinBloc? bloc;

  const SignConfirmPinPage({
    required this.onPinValidated,
    this.bloc,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return BlocProvider<PinBloc>(
      create: (BuildContext context) => bloc ?? PinBloc(context.read<AcceptSignAgreementUseCase>()),
      child: PinPage(
        headerBuilder: (context, attempts, isFinalRound) {
          final hasError = attempts != null;
          final String title, description;
          if (!hasError) {
            title = context.l10n.signConfirmPinPageTitle;
            description = context.l10n.signConfirmPinPageDescription;
          } else {
            title = context.l10n.signConfirmPinPageErrorTitle;
            description = context.l10n.signConfirmPinPageErrorDescription(attempts);
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
