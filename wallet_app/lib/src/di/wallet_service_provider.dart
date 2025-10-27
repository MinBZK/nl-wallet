import 'package:app_links/app_links.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../data/service/app_lifecycle_service.dart';
import '../data/service/auto_lock_service.dart';
import '../data/service/country_code_service.dart';
import '../data/service/deeplink_service.dart';
import '../data/service/event/app_event_coordinator.dart';
import '../data/service/event/listener/wallet_transfer_event_listener.dart';
import '../data/service/navigation_service.dart';
import '../data/service/semantics_event_service.dart';
import '../util/manager/biometric_unlock_manager.dart';

class WalletServiceProvider extends StatelessWidget {
  final Widget child;
  final GlobalKey<NavigatorState> navigatorKey;

  const WalletServiceProvider({
    required this.child,
    required this.navigatorKey,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return MultiRepositoryProvider(
      providers: [
        RepositoryProvider<AppLifecycleService>(
          create: (context) => AppLifecycleService(),
        ),
        RepositoryProvider<SemanticsEventService>(
          lazy: false,
          create: (context) => SemanticsEventService(),
        ),
        RepositoryProvider<CountryCodeService>(
          create: (context) => CountryCodeService(context.read()),
          lazy: false,
        ),
        RepositoryProvider<AutoLockService>(
          create: (context) => AutoLockService(),
        ),
        RepositoryProvider<NavigationService>(
          create: (context) => NavigationService(
            navigatorKey,
            context.read(),
            context.read(),
          ),
        ),
        RepositoryProvider<DeeplinkService>(
          create: (context) => DeeplinkService(
            AppLinks(),
            context.read(),
            context.read(),
            context.read(),
          ),
          lazy: false,
        ),
        RepositoryProvider<BiometricUnlockManager>(
          create: (context) => BiometricUnlockManager(
            context.read(),
            context.read(),
            context.read(),
          ),
          lazy: false,
        ),
        RepositoryProvider<AppEventCoordinator>(
          create: (context) => AppEventCoordinator(
            context.read(),
            [
              context.read<NavigationService>(),
              WalletTransferEventListener(
                context.read(),
                context.read(),
                context.read(),
              ),
            ],
          ),
          lazy: false,
        ),
      ],
      child: AppLifecycleObserver(
        child: child,
      ),
    );
  }
}
