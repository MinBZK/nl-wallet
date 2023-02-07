import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../feature/navigation/deeplink_service.dart';

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
        RepositoryProvider<DeeplinkService>(
          create: (context) => DeeplinkService(navigatorKey, context.read(), context.read()),
        )
      ],
      child: child,
    );
  }
}
