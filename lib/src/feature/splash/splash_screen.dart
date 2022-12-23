import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../wallet_routes.dart';
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
          if (state.isInitialized) {
            Navigator.restorablePushReplacementNamed(context, WalletRoutes.pinRoute);
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
                AppLocalizations.of(context).appTitle,
                style: Theme.of(context).textTheme.headline3,
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
