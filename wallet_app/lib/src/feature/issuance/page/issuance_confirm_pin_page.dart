import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/usecase/pin/confirm_transaction_usecase.dart';
import '../../common/widget/pin_header.dart';
import '../../pin/bloc/pin_bloc.dart';
import '../../pin/pin_page.dart';

class IssuanceConfirmPinPage extends StatelessWidget {
  final VoidCallback onPinValidated;

  const IssuanceConfirmPinPage({
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
            title = locale.issuanceConfirmPinPageTitle;
            description = locale.issuanceConfirmPinPageDescription;
          } else {
            title = locale.issuanceConfirmPinPageErrorTitle;
            description = locale.issuanceConfirmPinPageErrorDescription(attempts);
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
