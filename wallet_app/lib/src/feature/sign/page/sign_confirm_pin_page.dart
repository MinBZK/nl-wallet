import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/usecase/pin/confirm_transaction_usecase.dart';
import '../../common/widget/pin_header.dart';
import '../../pin/bloc/pin_bloc.dart';
import '../../pin/pin_page.dart';

class SignConfirmPinPage extends StatelessWidget {
  final VoidCallback onPinValidated;

  const SignConfirmPinPage({
    required this.onPinValidated,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return BlocProvider<PinBloc>(
      create: (BuildContext context) => PinBloc(context.read<ConfirmTransactionUseCase>(), context.read()),
      child: PinPage(
        headerBuilder: (context, attempts) {
          final hasError = attempts != null;
          final String title, description;
          if (!hasError) {
            title = locale.signConfirmPinPageTitle;
            description = locale.signConfirmPinPageDescription;
          } else {
            title = locale.signConfirmPinPageErrorTitle;
            description = locale.signConfirmPinPageErrorDescription(attempts);
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
