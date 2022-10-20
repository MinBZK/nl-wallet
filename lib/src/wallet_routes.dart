import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'feature/pin/pin_screen.dart';
import 'feature/splash/bloc/splash_bloc.dart';
import 'feature/splash/splash_screen.dart';

/// Class responsible for defining route names and for mapping these names to the actual
/// instantiation logic, this includes providing any optional dependencies (e.g. BLoCs).
class WalletRoutes {
  const WalletRoutes._();

  static const splashRoute = "/";
  static const pinRoute = "/pin";

  static const Map<String, WidgetBuilder> routes = {splashRoute: _createSplashRoute, pinRoute: _createPinRoute};
}

Widget _createSplashRoute(BuildContext context) => BlocProvider<SplashBloc>(
      create: (BuildContext context) => SplashBloc(context.read()),
      child: const SplashScreen(),
    );

Widget _createPinRoute(BuildContext context) => const PinScreen();
