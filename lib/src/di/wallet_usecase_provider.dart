import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../domain/usecase/app/check_is_app_initialized_usecase.dart';
import '../domain/usecase/pin/get_available_pin_attempts_usecase.dart';
import '../domain/usecase/pin/verify_wallet_pin_usecase.dart';

/// This widget is responsible for initializing and providing all Repositories.
/// Most likely to be used once at the top (app) level.
class WalletUseCaseProvider extends StatelessWidget {
  final Widget child;

  const WalletUseCaseProvider({required this.child, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MultiRepositoryProvider(
      providers: [
        RepositoryProvider<CheckIsAppInitializedUseCase>(
          create: (context) => CheckIsAppInitializedUseCase(context.read()),
        ),
        RepositoryProvider<VerifyWalletPinUseCase>(
          create: (context) => VerifyWalletPinUseCase(),
        ),
        RepositoryProvider<GetAvailablePinAttemptsUseCase>(
          create: (context) => GetAvailablePinAttemptsUseCase(),
        ),
      ],
      child: child,
    );
  }
}
