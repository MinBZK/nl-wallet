import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/card/wallet_card.dart';
import '../../../domain/usecase/issuance/impl/accept_issuance_usecase_impl.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/pin_header.dart';
import '../../pin/bloc/pin_bloc.dart';
import '../../pin/pin_page.dart';

class IssuanceConfirmPinForIssuancePage extends StatelessWidget {
  final OnPinValidatedCallback<String?> onPinValidated;
  final OnPinErrorCallback onConfirmWithPinFailed;
  final String? title;
  final List<WalletCard> cards;

  @visibleForTesting
  final PinBloc? bloc;

  const IssuanceConfirmPinForIssuancePage({
    required this.onPinValidated,
    required this.onConfirmWithPinFailed,
    required this.cards,
    this.title,
    this.bloc,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return BlocProvider<PinBloc>(
      create: (BuildContext context) => bloc ?? PinBloc(AcceptIssuanceUseCaseImpl(context.read(), cards: cards)),
      child: PinPage(
        headerBuilder: (context, attempts, isFinalRound) {
          return PinHeader(title: title ?? context.l10n.issuanceConfirmPinPageTitle);
        },
        onPinValidated: (result) => onPinValidated.call(result as String?),
        onPinError: onConfirmWithPinFailed,
      ),
    );
  }
}
