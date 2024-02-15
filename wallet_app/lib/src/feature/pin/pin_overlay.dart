import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/usecase/pin/unlock_wallet_with_pin_usecase.dart';
import 'bloc/pin_bloc.dart';
import 'pin_screen.dart';

/// Cache used as the [initialData] for the stream, this is
/// required to make sure [Hero] animations work as expected.
var _lockedStreamCache = true;

class PinOverlay extends StatelessWidget {
  final Widget child;
  final Stream<bool> isLockedStream;

  @visibleForTesting
  final PinBloc? bloc;

  const PinOverlay({
    required this.child,
    required this.isLockedStream,
    this.bloc,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return StreamBuilder<bool>(
      stream: isLockedStream,
      initialData: _lockedStreamCache,
      builder: (context, snapshot) {
        final isLocked = _lockedStreamCache = snapshot.data!;
        if (isLocked) {
          return BlocProvider<PinBloc>(
            create: (BuildContext context) => bloc ?? PinBloc(context.read<UnlockWalletWithPinUseCase>()),
            child: const PinScreen(),
          );
        } else {
          return child;
        }
      },
    );
  }
}
