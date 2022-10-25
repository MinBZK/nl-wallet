import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../data/repository/wallet/wallet_repository.dart';
import 'bloc/pin_bloc.dart';
import 'pin_screen.dart';

class PinOverlay extends StatelessWidget {
  final Widget child;

  const PinOverlay({required this.child, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return StreamBuilder<bool>(
      stream: context.read<WalletRepository>().isLockedStream,
      initialData: true,
      builder: (context, snapshot) {
        if (snapshot.data!) {
          return BlocProvider<PinBloc>(
            create: (BuildContext context) => PinBloc(context.read(), context.read()),
            child: const PinScreen(),
          );
        } else {
          return child;
        }
      },
    );
  }
}
