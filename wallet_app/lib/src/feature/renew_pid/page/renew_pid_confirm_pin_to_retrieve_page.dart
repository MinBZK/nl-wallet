import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/usecase/pid/issue_pid_for_review_usecase.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/pin_header.dart';
import '../../pin/bloc/pin_bloc.dart';
import '../../pin/pin_page.dart';

/// Asks for the PIN to retrieve the pid from an issuer that does not provide previews.
class RenewPidConfirmPinToRetrievePage extends StatelessWidget {
  final OnPinValidatedCallback<List<Attribute>> onPidRetrieved;

  /// Callback for when retrieving the pid fails with an unrecoverable error.
  final OnPinErrorCallback onRetrievePidFailed;

  @visibleForTesting
  final PinBloc? bloc;

  const RenewPidConfirmPinToRetrievePage({
    required this.onPidRetrieved,
    required this.onRetrievePidFailed,
    this.bloc,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return BlocProvider<PinBloc>(
      create: (BuildContext context) => bloc ?? PinBloc(context.read<IssuePidForReviewUseCase>()),
      child: PinPage(
        headerBuilder: (context, attempts, isFinalRound) => PinHeader(title: context.l10n.renewPidConfirmPinPageTitle),
        onPinValidated: (result) => onPidRetrieved(result as List<Attribute>),
        onPinError: onRetrievePidFailed,
      ),
    );
  }
}
