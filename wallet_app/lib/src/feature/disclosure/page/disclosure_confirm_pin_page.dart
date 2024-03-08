import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/disclosure/accept_disclosure_usecase.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/pin_header.dart';
import '../../pin/bloc/pin_bloc.dart';
import '../../pin/pin_page.dart';

class DisclosureConfirmPinPage extends StatelessWidget {
  final OnPinValidatedCallback onPinValidated;
  final OnPinErrorCallback onConfirmWithPinFailed;

  @visibleForTesting
  final PinBloc? bloc;

  const DisclosureConfirmPinPage({
    required this.onPinValidated,
    required this.onConfirmWithPinFailed,
    this.bloc,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return BlocProvider<PinBloc>(
      create: (BuildContext context) => bloc ?? PinBloc(context.read<AcceptDisclosureUseCase>()),
      child: PinPage(
        headerBuilder: (context, attempts, isFinalAttempt) {
          return PinHeader(title: context.l10n.disclosureConfirmPinPageTitle);
        },
        onPinValidated: onPinValidated,
        onPinError: onConfirmWithPinFailed,
      ),
    );
  }
}
