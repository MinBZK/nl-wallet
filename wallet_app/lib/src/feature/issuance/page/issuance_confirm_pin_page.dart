import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/pin/disclose_for_issuance_usecase.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/pin_header.dart';
import '../../pin/bloc/pin_bloc.dart';
import '../../pin/pin_page.dart';

class IssuanceConfirmPinPage extends StatelessWidget {
  final OnPinValidatedCallback onPinValidated;

  @visibleForTesting
  final PinBloc? bloc;

  const IssuanceConfirmPinPage({
    required this.onPinValidated,
    this.bloc,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return BlocProvider<PinBloc>(
      create: (BuildContext context) => bloc ?? PinBloc(context.read<DiscloseForIssuanceUseCase>()),
      child: PinPage(
        headerBuilder: (context, attempts, isFinalAttempt) {
          final hasError = attempts != null;
          final String title, description;
          if (!hasError) {
            title = context.l10n.issuanceConfirmPinPageTitle;
            description = context.l10n.issuanceConfirmPinPageDescription;
          } else {
            title = context.l10n.issuanceConfirmPinPageErrorTitle;
            description = context.l10n.issuanceConfirmPinPageErrorDescription(attempts);
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
