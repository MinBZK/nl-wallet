import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../common/widget/loading_indicator.dart';
import '../common/widget/wallet_logo.dart';
import 'bloc/splash_bloc.dart';

class SplashScreen extends StatelessWidget {
  const SplashScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BlocListener<SplashBloc, SplashState>(
      listenWhen: (prev, current) => current is SplashLoaded,
      listener: (context, state) {
        if (state is SplashLoaded) {
          if (state.hasPid && state.isRegistered) {
            Navigator.restorablePushReplacementNamed(context, WalletRoutes.homeRoute);
          } else if (state.isRegistered) {
            Navigator.restorablePushReplacementNamed(context, WalletRoutes.walletPersonalizeRoute);
          } else {
            Navigator.restorablePushReplacementNamed(context, WalletRoutes.introductionRoute);
          }
        }
      },
      child: Scaffold(
        body: Center(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.center,
            mainAxisSize: MainAxisSize.min,
            children: [
              const WalletLogo(size: 80),
              const SizedBox(height: 16),
              Text(
                context.l10n.appTitle,
                style: context.textTheme.displaySmall,
              ),
              const SizedBox(height: 16),
              const LoadingIndicator(),
            ],
          ),
        ),
      ),
    );
  }
}
