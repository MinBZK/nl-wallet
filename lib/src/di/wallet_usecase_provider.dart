import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../feature/splash/usecase/check_is_app_initialized_usecase.dart';

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
      ],
      child: child,
    );
  }
}
