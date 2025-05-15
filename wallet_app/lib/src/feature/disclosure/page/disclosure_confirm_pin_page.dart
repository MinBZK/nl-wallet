import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/disclosure/accept_disclosure_usecase.dart';
import '../../common/widget/pin_header.dart';
import '../../pin/bloc/pin_bloc.dart';
import '../../pin/pin_page.dart';

class DisclosureConfirmPinPage extends StatelessWidget {
  final OnPinValidatedCallback<String?> onPinValidated;
  final OnPinErrorCallback onConfirmWithPinFailed;
  final String title;

  @visibleForTesting
  final PinBloc? bloc;

  const DisclosureConfirmPinPage({
    required this.onPinValidated,
    required this.onConfirmWithPinFailed,
    required this.title,
    this.bloc,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return BlocProvider<PinBloc>(
      create: (BuildContext context) => bloc ?? PinBloc(context.read<AcceptDisclosureUseCase>()),
      child: PinPage(
        headerBuilder: (context, attempts, isFinalRound) {
          return PinHeader(title: title);
        },
        onPinValidated: (result) => onPinValidated.call(result as String?),
        onPinError: onConfirmWithPinFailed,
      ),
    );
  }
}
