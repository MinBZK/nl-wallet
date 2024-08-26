import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/pin/check_pin_usecase.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/pin_header.dart';
import '../../pin/bloc/pin_bloc.dart';
import '../../pin/pin_page.dart';

class EnterCurrentPinPage extends StatelessWidget {
  final Function(String) onPinValidated;

  @visibleForTesting
  final PinBloc? bloc;

  const EnterCurrentPinPage({required this.onPinValidated, this.bloc, super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider<PinBloc>(
      create: (BuildContext context) => bloc ?? PinBloc(context.read<CheckPinUseCase>()),
      child: Builder(
        // Builder to make sure the onPinValidated callback can access the [PinBloc].
        builder: (context) {
          return PinPage(
            headerBuilder: (context, attempts, isFinalRound) {
              return PinHeader(title: context.l10n.changePinScreenEnterCurrentPinTitle);
            },
            onPinValidated: (_) => onPinValidated(context.read<PinBloc>().currentPin),
          );
        },
      ),
    );
  }
}
