import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_native_splash/flutter_native_splash.dart';

import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../common/widget/utility/do_on_init.dart';
import '../common/widget/wallet_logo.dart';
import 'bloc/splash_bloc.dart';

class SplashScreen extends StatelessWidget {
  const SplashScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return DoOnInit(
      key: const Key('splash_screen_on_init'),
      onInit: (context) {
        FlutterNativeSplash.remove();
        WalletAssets.preloadPidSvgs();
      },
      child: BlocListener<SplashBloc, SplashState>(
        listenWhen: (prev, current) => current is SplashLoaded,
        listener: (context, state) {
          if (state is SplashLoaded) {
            if (state.hasPid && state.isRegistered) {
              Navigator.restorablePushReplacementNamed(context, WalletRoutes.dashboardRoute);
            } else if (state.isRegistered) {
              Navigator.restorablePushReplacementNamed(context, WalletRoutes.walletPersonalizeRoute);
            } else {
              Navigator.restorablePushReplacementNamed(context, WalletRoutes.introductionRoute);
            }
          }
        },
        child: _buildContent(context),
      ),
    );
  }

  /// Build the visual part of the SplashScreen
  Widget _buildContent(BuildContext context) {
    return Scaffold(
      key: const Key('splashScreen'),
      body: Center(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.center,
          mainAxisSize: MainAxisSize.min,
          children: [
            const WalletLogo(size: 100),
            const SizedBox(height: 16),
            Text(
              context.l10n.appTitle,
              style: context.textTheme.displaySmall,
            ),
          ],
        ),
      ),
    );
  }
}
