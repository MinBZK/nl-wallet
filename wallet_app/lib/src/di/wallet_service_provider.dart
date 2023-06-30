import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../data/service/app_lifecycle_service.dart';
import '../data/service/deeplink_service.dart';

class WalletServiceProvider extends StatelessWidget {
  final Widget child;
  final GlobalKey<NavigatorState> navigatorKey;

  const WalletServiceProvider({
    required this.child,
    required this.navigatorKey,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MultiRepositoryProvider(
      providers: [
        RepositoryProvider<AppLifecycleService>(
          create: (context) => AppLifecycleService(),
        ),
        RepositoryProvider<DeeplinkService>(
          create: (context) => DeeplinkService(
            navigatorKey,
            context.read(),
            context.read(),
            context.read(),
            context.read(),
            context.read(),
            context.read(),
            context.read(),
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
