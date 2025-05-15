import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/card/wallet_card.dart';
import '../../../domain/usecase/pin/disclose_for_issuance_usecase.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/pin_header.dart';
import '../../pin/bloc/pin_bloc.dart';
import '../../pin/pin_page.dart';

class IssuanceConfirmPinForDisclosurePage extends StatelessWidget {
  final OnPinValidatedCallback<List<WalletCard>> onPinValidated;
  final OnPinErrorCallback onConfirmWithPinFailed;
  final String? title;

  @visibleForTesting
  final PinBloc? bloc;

  const IssuanceConfirmPinForDisclosurePage({
    required this.onPinValidated,
    required this.onConfirmWithPinFailed,
    this.title,
    this.bloc,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return BlocProvider<PinBloc>(
      create: (BuildContext context) => bloc ?? PinBloc(context.read<DiscloseForIssuanceUseCase>()),
      child: PinPage(
        headerBuilder: (context, attempts, isFinalRound) {
          return PinHeader(title: title ?? context.l10n.issuanceConfirmPinPageTitle);
        },
        onPinValidated: (result) => onPinValidated.call(result as List<WalletCard>),
        onPinError: onConfirmWithPinFailed,
      ),
    );
  }
}
